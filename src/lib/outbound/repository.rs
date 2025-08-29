use crate::domain::petstore::ports::PetRepository;
use crate::domain::petstore::models::pet::{Pet, CreatePetRequest, CreatePetError, Status};
use crate::domain::petstore::models::category::Category;
use crate::domain::petstore::models::tag::Tag;
use crate::outbound::connect::PostgresClient;
use sqlx::Row;

#[derive(serde::Deserialize)]
struct TagData {
    id: Option<i64>,
    name: Option<String>,
}

impl PetRepository for PostgresClient {
    async fn add_pet(&self, req: &CreatePetRequest) -> Result<Pet, CreatePetError> {
        // Check for duplicate pet name
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pets WHERE name = $1)")
            .bind(&req.name)
            .fetch_one(self.pool())
            .await
            .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

        if exists {
            return Err(CreatePetError::Duplicate { name: req.name.clone() });
        }

        // Insert category if provided
        let category_id = if let Some(category) = &req.category {
            let cat_id: i64 = sqlx::query_scalar(
                "INSERT INTO categories (id, name) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name RETURNING id"
            )
            .bind(category.id)
            .bind(&category.name)
            .fetch_one(self.pool())
            .await
            .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;
            Some(cat_id)
        } else {
            None
        };

        // Insert the pet
        let pet_id: i64 = sqlx::query_scalar(
            "INSERT INTO pets (id, name, category_id, status) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&req.id)
        .bind(&req.name)
        .bind(category_id)
        .bind(req.status.as_ref().map(|s| s.to_str()))
        .fetch_one(self.pool())
        .await
        .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

        // Insert photo URLs if any
        if !req.photo_urls.is_empty() {
            for url in &req.photo_urls {
                sqlx::query("INSERT INTO pet_photos (pet_id, url) VALUES ($1, $2)")
                    .bind(pet_id)
                    .bind(url)
                    .execute(self.pool())
                    .await
                    .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;
            }
        }

        // Insert tags if any
        if !req.tags.is_empty() {
            for tag in &req.tags {
                // First ensure tag exists
                let tag_id: i64 = sqlx::query_scalar(
                    "INSERT INTO tags (id, name) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id"
                )
                .bind(&tag.id)
                .bind(&tag.name)
                .fetch_one(self.pool())
                .await
                .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

                // Then link tag to pet
                sqlx::query("INSERT INTO pet_tags (pet_id, tag_id) VALUES ($1, $2)")
                    .bind(pet_id)
                    .bind(tag_id)
                    .execute(self.pool())
                    .await
                    .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;
            }
        }

        // Create and return the pet
        let mut pet = Pet::new(req.name.clone());
        pet.id = Some(pet_id);
        if let Some(category) = &req.category {
            pet.set_category(category.clone());
        }
        for url in &req.photo_urls {
            pet.add_photo(url.clone());
        }
        for tag in &req.tags {
            pet.add_tag(tag.clone());
        }
        if let Some(status) = &req.status {
            pet.set_status(status.clone());
        }

        Ok(pet)
    }

    async fn find_pet_by_id(&self, pet_id: i64) -> Result<Option<Pet>, CreatePetError> {
        // Get pet details
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.name, p.status, c.id as category_id, c.name as category_name
            FROM pets p
            LEFT JOIN categories c ON p.category_id = c.id
            WHERE p.id = $1
            "#
        )
        .bind(pet_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

        let Some(row) = rows else {
            return Ok(None);
        };

        // Build the pet
        let mut pet = Pet::new(row.get::<String, _>("name"));
        pet.id = Some(row.get::<i64, _>("id"));
        let status = match row.get::<String, _>("status").as_str() {
            "available" => Status::Available,
            "pending" => Status::Pending,
            "sold" => Status::Sold,
            _ => Status::Available,
        };
        pet.set_status(status);

        // Set category if available
        if let (Ok(category_id), Ok(category_name)) = (
            row.try_get::<i64, _>("category_id"),
            row.try_get::<String, _>("category_name")
        ) {
            pet.set_category(Category::with_values(category_id, category_name));
        }

        // Get photo URLs
        let photo_urls: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT url FROM pet_photos WHERE pet_id = $1"
        )
        .bind(pet_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

        // Get tags
        let tags: Vec<String> = sqlx::query_scalar::<_, String>(
            r#"
            SELECT json_build_object('id', t.id, 'name', t.name)::text
            FROM tags t
            JOIN pet_tags pt ON t.id = pt.tag_id
            WHERE pt.pet_id = $1
            "#
        )
        .bind(pet_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

        let tags: Vec<TagData> = tags.into_iter()
            .map(|json_str| serde_json::from_str(&json_str))
            .collect::<Result<_, _>>()
            .map_err(|e| CreatePetError::Unknown(anyhow::anyhow!(e)))?;

        // Add photos and tags
        for url in photo_urls {
            pet.add_photo(url);
        }
        for tag in tags {
            if let (Some(id), Some(name)) = (tag.id, tag.name) {
                pet.add_tag(Tag::with_values(id, name));
            }
        }

        Ok(Some(pet))
    }
}
