use derive_more::From;
use thiserror::Error;


use super::category::Category;
use super::tag::Tag;


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pet {
    pub id: Option<i64>,
    pub name: String,
    pub category: Option<Box<Category>>,
    pub photo_urls: Vec<String>,
    pub tags: Vec<Tag>,
    pub status: Option<Status>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    Available,
    Pending,
    Sold,
}

impl Status {
    pub fn to_str(&self) -> &str {
        match self {
            Status::Available => "available",
            Status::Pending => "pending",
            Status::Sold => "sold",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Default for Status {
    fn default() -> Status {
        Self::Available
    }
}

impl Pet {
    pub fn new(name: String) -> Self {
        Pet {
            id: None,
            name,
            category: None,
            photo_urls: Vec::new(),
            tags: Vec::new(),
            status: Some(Status::default()),
        }
    }

    pub fn with_id(id: i64, name: String) -> Self {
        Pet {
            id: Some(id),
            name,
            category: None,
            photo_urls: Vec::new(),
            tags: Vec::new(),
            status: Some(Status::default()),
        }
    }

    pub fn set_category(&mut self, category: Category) {
        self.category = Some(Box::new(category));
    }

    pub fn add_photo(&mut self, url: String) {
        self.photo_urls.push(url);
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct CreatePetRequest {
    pub id: Option<i64>,
    pub name: String,
    pub category: Option<Category>,
    pub photo_urls: Vec<String>,
    pub tags: Vec<Tag>,
    pub status: Option<Status>,
}

impl CreatePetRequest {
    pub fn new(id: Option<i64>, name: String, category: Option<Category>, photo_urls: Vec<String>, tags: Vec<Tag>, status: Option<Status>) -> Self {
        Self {
            id,
            name,
            category,
            photo_urls,
            tags,   
            status,
        }
    }
 
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &Option<Category> {
        &self.category
    }

    pub fn photo_urls(&self) -> &Vec<String> {
        &self.photo_urls
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn status(&self) -> &Option<Status> {
        &self.status
    }
}

#[derive(Debug, Error)]
pub enum CreatePetError {
    #[error("pet with name {name} already exists")]
    Duplicate { name: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pet() {
        let name = String::from("Fluffy");
        let pet = Pet::new(name.clone());

        assert!(pet.id.is_none());
        assert_eq!(pet.name, name);
        assert!(pet.category.is_none());
        assert!(pet.photo_urls.is_empty());
        assert!(pet.tags.is_empty());
        assert!(matches!(pet.status, Some(Status::Available)));
    }

    #[test]
    fn test_pet_with_id() {
        let id = 1;
        let name = String::from("Rex");
        let pet = Pet::with_id(id, name.clone());

        assert_eq!(pet.id, Some(id));
        assert_eq!(pet.name, name);
        assert!(pet.category.is_none());
        assert!(pet.photo_urls.is_empty());
        assert!(pet.tags.is_empty());
        assert!(matches!(pet.status, Some(Status::Available)));
    }

    #[test]
    fn test_pet_with_category() {
        let name = String::from("Buddy");
        let mut pet = Pet::new(name);
        
        let category = Category::with_values(1, String::from("Dogs"));
        pet.set_category(category.clone());

        assert!(pet.category.is_some());
        let boxed_category = pet.category.unwrap();
        assert_eq!(*boxed_category, category);
    }

    #[test]
    fn test_pet_with_photos() {
        let name = String::from("Max");
        let mut pet = Pet::new(name);
        
        pet.add_photo(String::from("http://example.com/photo1.jpg"));
        pet.add_photo(String::from("http://example.com/photo2.jpg"));

        assert_eq!(pet.photo_urls.len(), 2);
        assert_eq!(pet.photo_urls[0], "http://example.com/photo1.jpg");
        assert_eq!(pet.photo_urls[1], "http://example.com/photo2.jpg");
    }

    #[test]
    fn test_pet_with_tags() {
        let name = String::from("Charlie");
        let mut pet = Pet::new(name);
        
        let tag = Tag::with_values(1, String::from("Friendly"));
        pet.add_tag(tag.clone());

        assert_eq!(pet.tags.len(), 1);
        assert_eq!(pet.tags[0], tag);
    }

    #[test]
    fn test_pet_status_change() {
        let name = String::from("Luna");
        let mut pet = Pet::new(name);
        
        assert!(matches!(pet.status, Some(Status::Available)));
        
        pet.set_status(Status::Pending);
        assert!(matches!(pet.status, Some(Status::Pending)));
        
        pet.set_status(Status::Sold);
        assert!(matches!(pet.status, Some(Status::Sold)));
    }

    #[test]
    fn test_pet_clone() {
        let mut pet = Pet::new(String::from("Bella"));
        pet.set_category(Category::with_values(1, String::from("Dogs")));
        pet.add_photo(String::from("http://example.com/bella.jpg"));
        pet.add_tag(Tag::with_values(1, String::from("Playful")));
        
        let cloned = pet.clone();
        assert_eq!(pet, cloned);
    }

    #[test]
    fn test_pet_ordering() {
        let pet1 = Pet::with_id(1, String::from("Ace"));
        let pet2 = Pet::with_id(2, String::from("Buddy"));
        
        assert!(pet1 < pet2); // Orders by id first
    }

    #[test]
    fn test_create_pet_request() {
        let id = Some(1);
        let name: String = String::from("Buddy");
        let category = Some(Category::with_values(1, String::from("Dogs")));
        let photo_urls = vec![String::from("http://example.com/buddy.jpg")];
        let tags = vec![Tag::with_values(1, String::from("Friendly"))];
        let status = Some(Status::Available);

        let request = CreatePetRequest::new(
            id.clone(),
            name.clone(),
            category.clone(),
            photo_urls.clone(),
            tags.clone(),
            status.clone(),
        );

        // Test getters
        assert_eq!(request.name(), name);
        assert_eq!(request.category(), &category);
        assert_eq!(request.photo_urls(), &photo_urls);
        assert_eq!(request.tags(), &tags);
        assert_eq!(request.status(), &status);
    }

    #[test]
    fn test_create_pet_request_minimal() {
        let name = String::from("Kitty");
        let request = CreatePetRequest::new(
            None,
            name.clone(),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );

        assert_eq!(request.name(), name);
        assert!(request.category().is_none());
        assert!(request.photo_urls().is_empty());
        assert!(request.tags().is_empty());
        assert!(request.status().is_none());
    }

    #[test]
    fn test_create_pet_request_from_struct() {
        let request = CreatePetRequest {
            id: None,
            name: String::from("Max"),
            category: Some(Category::with_values(2, String::from("Cats"))),
            photo_urls: vec![String::from("http://example.com/max.jpg")],
            tags: vec![
                Tag::with_values(1, String::from("Playful")),
                Tag::with_values(2, String::from("Young")),
            ],
            status: Some(Status::Pending),
        };

        assert_eq!(request.name(), "Max");
        assert!(request.category().is_some());
        assert_eq!(request.photo_urls().len(), 1);
        assert_eq!(request.tags().len(), 2);
        assert_eq!(request.status(), &Some(Status::Pending));
    }

    #[test]
    fn test_create_pet_error_duplicate() {
        let name = String::from("Rex");
        let error = CreatePetError::Duplicate { name: name.clone() };
        
        assert_eq!(
            error.to_string(),
            format!("pet with name {} already exists", name)
        );
    }

    #[test]
    fn test_create_pet_error_unknown() {
        let source = anyhow::anyhow!("database connection failed");
        let error = CreatePetError::Unknown(source);
        
        // Test the Display implementation
        assert!(error.to_string().contains("database connection failed"));
    }

    #[test]
    fn test_create_pet_error_debug() {
        let error = CreatePetError::Duplicate { 
            name: String::from("Max") 
        };
        
        // Verify Debug implementation
        assert!(format!("{:?}", error).contains("Duplicate"));
        assert!(format!("{:?}", error).contains("Max"));
    }
}

