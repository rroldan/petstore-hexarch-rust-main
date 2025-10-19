/*
   Module `find_pet_by_id` specifies an HTTP handler for finding a [Pet] by its ID.
*/

use axum::extract::{State, Path};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::domain::petstore::models::pet::Pet;
use crate::domain::petstore::ports::PetService;
use crate::inbound::http::AppState;



#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FindPetResponseData {
    Success {
        id: Option<i64>,
        name: String,
        category: Option<Box<crate::domain::petstore::models::category::Category>>,
        photo_urls: Vec<String>,
        tags: Option<Vec<crate::domain::petstore::models::tag::Tag>>,
        status: Option<String>,
    },
    Error {
        message: String,
    },
}

impl From<&Pet> for FindPetResponseData {
    fn from(pet: &Pet) -> Self {
        Self::Success {
            id: pet.id,
            name: pet.name.clone(),
            category: pet.category.clone(),
            photo_urls: pet.photo_urls.clone(),
            tags: Some(pet.tags.clone()),
            status: pet.status.as_ref().map(|s| s.to_string()),
        }
    }
}

/// Find a [Pet] by its ID.
///
/// # Responses
///
/// - 200 OK: the [Pet] was found.
/// - 404 Not Found: no [Pet] exists with the given ID.
/// - 500 Internal Server Error: an unexpected error occurred.
pub async fn find_pet_by_id<BS: PetService>(
    State(state): State<AppState<BS>>,
    Path(pet_id): Path<i64>,
) -> Result<(StatusCode, Json<crate::inbound::http::handlers::add_pet::ApiResponseBody<FindPetResponseData>>), crate::inbound::http::handlers::add_pet::ApiError> {
    let pet = state
        .pet_service
        .find_pet_by_id(pet_id)
        .await
        .map_err(crate::inbound::http::handlers::add_pet::ApiError::from)?;

    match pet {
        Some(pet) => Ok((
            StatusCode::OK,
            Json(crate::inbound::http::handlers::add_pet::ApiResponseBody::new(
                StatusCode::OK,
                FindPetResponseData::from(&pet),
            )),
        )),
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(crate::inbound::http::handlers::add_pet::ApiResponseBody::new(
                StatusCode::NOT_FOUND,
                FindPetResponseData::Error {
                    message: format!("pet with id {} not found", pet_id),
                },
            )),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use axum::http::StatusCode;
    use crate::domain::petstore::models::pet::{Pet, CreatePetError, Status};
    use crate::domain::petstore::models::category::Category;
    use crate::domain::petstore::models::tag::Tag;
    use crate::domain::petstore::ports::PetService;
    use super::*;

    #[derive(Clone)]
    struct MockPetService {
        find_pet_result: Arc<std::sync::Mutex<Option<Result<Option<Pet>, CreatePetError>>>>,
    }

    impl PetService for MockPetService {
        async fn add_pet(
            &self,
            _: &crate::domain::petstore::models::pet::CreatePetRequest,
        ) -> Result<Pet, CreatePetError> {
            Err(CreatePetError::Unknown(anyhow::anyhow!("Not implemented")))
        }

        async fn find_pet_by_id(
            &self,
            _: i64,
        ) -> Result<Option<Pet>, CreatePetError> {
            let mut guard = self.find_pet_result.lock().unwrap();
            guard.take().unwrap_or_else(|| Err(CreatePetError::Unknown(anyhow::anyhow!("Mock find_pet_by_id result not set"))))
        }
    }

    fn create_mock_pet() -> Pet {
        let mut pet = Pet::new(String::from("doggie"));
        pet.id = Some(10);
        pet.set_category(Category::with_values(1, String::from("Dogs")));
        pet.add_photo(String::from("string"));
        pet.add_tag(Tag::with_values(0, String::from("string")));
        pet.set_status(Status::Available);
        pet
    }

    #[tokio::test]
    async fn test_find_pet_by_id_success() {
        // Arrange
        let pet = create_mock_pet();
        let service = MockPetService {
            find_pet_result: Arc::new(std::sync::Mutex::new(Some(Ok(Some(pet.clone()))))),
        };
        
        let state = axum::extract::State(crate::inbound::http::AppState {
            pet_service: Arc::new(service),
        });

        let expected = (
            StatusCode::OK,
            Json(crate::inbound::http::handlers::add_pet::ApiResponseBody::new(
                StatusCode::OK,
                FindPetResponseData::Success {
                    id: pet.id,
                    name: pet.name,
                    category: pet.category,
                    photo_urls: pet.photo_urls,
                    tags: Some(pet.tags),
                    status: pet.status.map(|s| s.to_string()),
                },
            )),
        );

        // Act
        let actual = find_pet_by_id(state, axum::extract::Path(10)).await;

        // Assert
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.0, expected.0);
        assert_eq!(actual.1 .0, expected.1 .0);
    }

    #[tokio::test]
    async fn test_find_pet_by_id_not_found() {
        // Arrange
        let service = MockPetService {
            find_pet_result: Arc::new(std::sync::Mutex::new(Some(Ok(None)))),
        };
        
        let state = axum::extract::State(crate::inbound::http::AppState {
            pet_service: Arc::new(service),
        });

        // Act
        let actual = find_pet_by_id(state, axum::extract::Path(999)).await;

        // Assert
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.0, StatusCode::NOT_FOUND);
        assert!(matches!(actual.1.0, crate::inbound::http::handlers::add_pet::ApiResponseBody { .. }));
    }

    #[tokio::test]
    async fn test_find_pet_by_id_error() {
        // Arrange
        let service = MockPetService {
            find_pet_result: Arc::new(std::sync::Mutex::new(Some(Err(CreatePetError::Unknown(
                anyhow::anyhow!("database error"),
            ))))),
        };
        
        let state = axum::extract::State(crate::inbound::http::AppState {
            pet_service: Arc::new(service),
        });

        // Act
        let actual = find_pet_by_id(state, axum::extract::Path(10)).await;

        // Assert
        assert!(actual.is_err());
        let error = actual.unwrap_err();
        assert!(matches!(error, crate::inbound::http::handlers::add_pet::ApiError::InternalServerError(_)));
    }
} 