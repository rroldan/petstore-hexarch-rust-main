/*
   Module `ports` specifies the API by which external modules interact with the domain.

   All traits are bounded by `Send + Sync + 'static`, since their implementations must be shareable
   between request-handling threads.

   Trait methods are explicitly asynchronous, including `Send` bounds on response types,
   since the application is expected to always run in a multithreaded environment.
*/

use std::future::Future;
use crate::domain::petstore::models::pet::{Pet, CreatePetRequest, CreatePetError};

/// `PetService` is the public API for the pet domain.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait PetService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [Pet].
    ///
    /// # Errors:
    ///
    /// - [CreateAuthorError::Duplicate] if an [Pet] with the same [name] already exists.
    fn add_pet(
        &self,
        req: &CreatePetRequest,
    ) -> impl Future<Output = Result<Pet, CreatePetError>> + Send;

    /// Find a pet by its ID.
    ///
    /// # Errors:
    ///
    /// - Propagates any [CreatePetError] returned by the [PetRepository].
    fn find_pet_by_id(
        &self,
        pet_id: i64,
    ) -> impl Future<Output = Result<Option<Pet>, CreatePetError>> + Send;
}

/// `PetRepository` represents a store of pet data.
///
/// External modules must conform to this contract – the domain is not concerned with the
/// implementation details or underlying technology of any external code.
pub trait PetRepository: Send + Sync + Clone + 'static {
    /// Asynchronously persist a new [Author].
    ///
    /// # Errors:
    ///
    /// - MUST return [CreateAuthorError::Duplicate] if an [Pet]] with the same [name]]
    ///   already exists.
    fn add_pet(
        &self,
        req: &CreatePetRequest,
    ) -> impl Future<Output = Result<Pet, CreatePetError>> + Send;

    /// Find a pet by its ID.
    ///
    /// # Errors:
    ///
    /// - Propagates any [CreatePetError] returned by the database.
    fn find_pet_by_id(
        &self,
        pet_id: i64,
    ) -> impl Future<Output = Result<Option<Pet>, CreatePetError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock implementation of PetRepository for testing
    #[derive(Clone)]
    struct MockPetRepository {
        pets: Arc<Mutex<HashMap<String, Pet>>>,
    }

    impl MockPetRepository {
        fn new() -> Self {
            Self {
                pets: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl PetRepository for MockPetRepository {
        fn add_pet(
            &self,
            req: &CreatePetRequest,
        ) -> impl Future<Output = Result<Pet, CreatePetError>> + Send {
            let pets = self.pets.clone();
            let req = req.clone();
            
            async move {
                let mut pets = pets.lock().unwrap();
                
                // Check for duplicate
                if pets.contains_key(&req.name) {
                    return Err(CreatePetError::Duplicate { 
                        name: req.name 
                    });
                }

                // Create new pet
                let pet = Pet::new(req.name.clone());
                pets.insert(req.name, pet.clone());
                Ok(pet)
            }
        }

        fn find_pet_by_id(
            &self,
            pet_id: i64,
        ) -> impl Future<Output = Result<Option<Pet>, CreatePetError>> + Send {
            let pets = self.pets.clone();
            
            async move {
                let pets = pets.lock().unwrap();
                let pet = pets.values().find(|p| p.id == Some(pet_id)).cloned();
                Ok(pet)
            }
        }
    }

    // Mock implementation of PetService for testing
    #[derive(Clone)]
    struct MockPetService {
        repository: MockPetRepository,
    }

    impl MockPetService {
        fn new(repository: MockPetRepository) -> Self {
            Self { repository }
        }
    }

    impl PetService for MockPetService {
        fn add_pet(
            &self,
            req: &CreatePetRequest,
        ) -> impl Future<Output = Result<Pet, CreatePetError>> + Send {
            self.repository.add_pet(req)
        }

        fn find_pet_by_id(
            &self,
            pet_id: i64,
        ) -> impl Future<Output = Result<Option<Pet>, CreatePetError>> + Send {
            self.repository.find_pet_by_id(pet_id)
        }
    }

    #[tokio::test]
    async fn test_add_pet_success() {
        let repository = MockPetRepository::new();
        let service = MockPetService::new(repository);

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
    async fn test_add_pet_duplicate() {
        let repository = MockPetRepository::new();
        let service = MockPetService::new(repository);

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
    async fn test_repository_add_pet() {
        let repository = MockPetRepository::new();

        let request = CreatePetRequest::new(
            None,
            String::from("Luna"),
            None,
            Vec::new(),
            Vec::new(),
            None,
        );

        // Test successful addition
        let result = repository.add_pet(&request).await;
        assert!(result.is_ok());
        
        // Test duplicate error
        let result = repository.add_pet(&request).await;
        assert!(matches!(
            result,
            Err(CreatePetError::Duplicate { name }) if name == "Luna"
        ));
    }
}
