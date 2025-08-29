/*!
   Module `service` provides the canonical implementation of the [PetService] port. All
   blog-domain logic is defined here.
*/

use crate::domain::petstore::models::pet::{Pet, CreatePetRequest, CreatePetError};
use crate::domain::petstore::ports::{PetRepository, PetService};

/// Canonical implementation of the [PetService] port, through which the pet domain API is
/// consumed.
#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: PetRepository,
{
    repo: R
}

impl<R> Service<R>
where
    R: PetRepository
{
    pub fn new(repo: R) -> Self {
        Self {
            repo
        }
    }
}

impl<R> PetService for Service<R>
where
    R: PetRepository
{
    /// Create the [Pet] specified in `req` and trigger notifications.
    ///
    /// # Errors:
    ///
    /// - Propagates any [CreatePetError] returned by the [PetRepository].
    async fn add_pet(&self, req: &CreatePetRequest) -> Result<Pet, CreatePetError> {
        let result = self.repo.add_pet(req).await;
        result
    }

    /// Find a pet by its ID.
    ///
    /// # Errors:
    ///
    /// - Propagates any [CreatePetError] returned by the [PetRepository].
    async fn find_pet_by_id(&self, pet_id: i64) -> Result<Option<Pet>, CreatePetError> {
        self.repo.find_pet_by_id(pet_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use crate::domain::petstore::models::pet::Status;
    use crate::domain::petstore::models::category::Category;
    use crate::domain::petstore::models::tag::Tag;

    // Mock implementation of PetRepository for testing
    #[derive(Debug, Clone)]
    struct MockRepository {
        pets: Arc<Mutex<HashMap<String, Pet>>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                pets: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl PetRepository for MockRepository {
        async fn add_pet(&self, req: &CreatePetRequest) -> Result<Pet, CreatePetError> {
            let mut pets = self.pets.lock().unwrap();
            
            // Check for duplicate
            if pets.contains_key(&req.name) {
                return Err(CreatePetError::Duplicate { 
                    name: req.name.clone() 
                });
            }

            // Create new pet
            let mut pet = Pet::new(req.name.clone());
            if let Some(id) = req.id {
                pet.id = Some(id);
            }
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
            pets.insert(req.name.clone(), pet.clone());
            Ok(pet)
        }

        async fn find_pet_by_id(&self, pet_id: i64) -> Result<Option<Pet>, CreatePetError> {
            let pets = self.pets.lock().unwrap();
            let pet = pets.values().find(|p| p.id == Some(pet_id)).cloned();
            Ok(pet)
        }
    }

    #[tokio::test]
    async fn test_service_new() {
        let repo = MockRepository::new();
        let service = Service::new(repo);
        
        // Verify service was created (using debug print)
        assert!(format!("{:?}", service).contains("Service"));
    }

    #[tokio::test]
    async fn test_service_add_pet_success() {
        let repo = MockRepository::new();
        let service = Service::new(repo);

        let request = CreatePetRequest::new(
            None,
            String::from("Buddy"),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );

        let result = service.add_pet(&request).await;
        assert!(result.is_ok());
        
        let pet = result.unwrap();
        assert_eq!(pet.name, "Buddy");
        assert!(pet.id.is_none());
        assert!(pet.category.is_none());
        assert!(pet.photo_urls.is_empty());
        assert!(pet.tags.is_empty());
        assert!(matches!(pet.status, Some(crate::domain::petstore::models::pet::Status::Available)));
    }

    #[tokio::test]
    async fn test_service_add_pet_duplicate() {
        let repo = MockRepository::new();
        let service = Service::new(repo);

        let request = CreatePetRequest::new(
            None,
            String::from("Max"),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );

        // First addition should succeed
        let result = service.add_pet(&request).await;
        assert!(result.is_ok());

        // Second addition should fail with Duplicate error
        let result = service.add_pet(&request).await;
        assert!(matches!(
            result,
            Err(CreatePetError::Duplicate { name }) if name == "Max"
        ));
    }

    #[tokio::test]
    async fn test_service_find_pet_by_id() {
        let repo = MockRepository::new();
        let service = Service::new(repo);

        // Add a pet first
        let request = CreatePetRequest::new(
            Some(1),
            String::from("Buddy"),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );

        let pet = service.add_pet(&request).await.unwrap();
        assert_eq!(pet.id, Some(1));

        // Test finding the pet
        let found = service.find_pet_by_id(1).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "Buddy");
        assert_eq!(found.id, Some(1));

        // Test finding non-existent pet
        let not_found = service.find_pet_by_id(999).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_service_clone() {
        let repo = MockRepository::new();
        let service = Service::new(repo);
        let cloned = service.clone();
        
        // First request with original service
        let request = CreatePetRequest::new(
            None,
            String::from("Luna"),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );

        let result1 = service.add_pet(&request).await;
        assert!(result1.is_ok()); // First addition should succeed
        
        // Same request with cloned service should fail (duplicate)
        let result2 = cloned.add_pet(&request).await;
        assert!(result2.is_err()); // Should fail as pet was already added

        // Different pet with cloned service should succeed
        let request3 = CreatePetRequest::new(
            None,
            String::from("Rex"),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );
        
        let result3 = cloned.add_pet(&request3).await;
        assert!(result3.is_ok()); // Should succeed as it's a new pet
    }

    #[tokio::test]
    async fn test_service_find_pet_by_id_with_all_fields() {
        let repo = MockRepository::new();
        let service = Service::new(repo);

        // Add a pet with all fields
        let category = Category::with_values(1, "Dogs".to_string());
        let tags = vec![
            Tag::with_values(0, "string".to_string()),
        ];
        let photo_urls = vec!["string".to_string()];
        let status = Status::Available;

        let request = CreatePetRequest::new(
            Some(10),
            "doggie".to_string(),
            Some(category),
            photo_urls.clone(),
            tags.clone(),
            Some(status),
        );

        // Add the pet
        let pet = service.add_pet(&request).await.unwrap();
        assert_eq!(pet.id, Some(10));
        assert_eq!(pet.name, "doggie");
        assert!(pet.category.is_some());
        assert_eq!(pet.photo_urls, photo_urls);
        assert_eq!(pet.tags, tags);
        assert_eq!(pet.status, Some(Status::Available));

        // Test finding the pet
        let found = service.find_pet_by_id(10).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, Some(10));
        assert_eq!(found.name, "doggie");
        assert!(found.category.is_some());
        let category = found.category.unwrap();
        assert_eq!(category.id, Some(1));
        assert_eq!(category.name, Some("Dogs".to_string()));
        assert_eq!(found.photo_urls, photo_urls);
        assert_eq!(found.tags, tags);
        assert_eq!(found.status, Some(Status::Available));
    }
}