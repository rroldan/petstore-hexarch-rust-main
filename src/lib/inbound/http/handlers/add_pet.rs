/*
   Module `add_pet` specifies an HTTP handler for creating a new [Pet], and the
   associated data structures.
*/

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::petstore::models::pet::{Pet, CreatePetRequest, CreatePetError, Status};
use crate::domain::petstore::models::category::Category;
use crate::domain::petstore::models::tag::Tag;      
use crate::domain::petstore::models::value_objects::{CategoryError, PetName, PetNameError, PhotoUrls, PhotoUrlsError, StatusError, Tags, TagsError};

use crate::domain::petstore::ports::PetService;
use crate::inbound::http::AppState;

#[derive(Debug, Clone)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T> PartialEq for ApiSuccess<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 .0 == other.1 .0
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
    }
}

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
    BadRequest(String),
}

impl ApiError {
    // pub fn status_code(&self) -> StatusCode {
    //     match self {
    //         ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    //         ApiError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
    //         ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
    //     }
    // }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::UnprocessableEntity(msg) => write!(f, "Unprocessable Entity: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

impl From<CreatePetError> for ApiError {
    fn from(e: CreatePetError) -> Self {
        match e {
            CreatePetError::Duplicate { name } => {
                Self::UnprocessableEntity(format!("pet with name {} already exists", name))
            }
            CreatePetError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<ParseCreatePetHttpRequestError> for ApiError {
    fn from(e: ParseCreatePetHttpRequestError) -> Self {
        let message = match e {
            ParseCreatePetHttpRequestError::Name(cause) => {
                format!("pet name {} is invalid", cause)
            }
            ParseCreatePetHttpRequestError::Category(cause) => {
                format!("category {} is invalid", cause)
            }
            ParseCreatePetHttpRequestError::PhotoUrls(cause) => {
                format!("photo urls {} is invalid", cause)
            }
            ParseCreatePetHttpRequestError::Tags(cause) => {
                format!("tags {} is invalid", cause)
            }
            ParseCreatePetHttpRequestError::Status(cause) => {
                format!("status {} is invalid", cause)
            }
        };

        Self::BadRequest(message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use ApiError::*;

        match self {
            InternalServerError(e) => {
                tracing::error!("{}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponseBody::new_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )),
                )
                    .into_response()
            }
            UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponseBody::new_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    message,
                )),
            )
                .into_response(),
            BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponseBody::new_error(
                    StatusCode::BAD_REQUEST,
                    message,
                )),
            )
                .into_response(),
        }
    }
}

/// Generic response structure shared by all API responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    pub fn new(status_code: StatusCode, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

/// The response data format for all error responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorData {
    pub message: String,
}

/// The body of an [Pet] creation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreatePetRequestBody {
    pub id: Option<i64>,
    pub name: String,
    pub category: Option<Category>,
    pub photo_urls: Vec<String>,
    pub tags: Option<Vec<Tag>>,
    pub status: Option<String>,
}

/// The response body data field for successful [Pet] creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatePetResponseData {
    pub id: Option<i64>,
    pub name: String,
    pub category: Option<Category>,
    pub photo_urls: Vec<String>,
    pub tags: Option<Vec<Tag>>,
    pub status: Option<String>, 
}

impl From<&Pet> for CreatePetResponseData {
    fn from(pet: &Pet) -> Self {
        Self {
            id: pet.id,
            name: pet.name.clone(),
            category: pet.category.as_ref().map(|c| Category {
                id: c.id,
                name: c.name.clone()
            }),
            photo_urls: pet.photo_urls.clone(),
            tags: Some(pet.tags.clone()),
            status: pet.status.as_ref().map(|s| s.to_string()),
        }
    }
}

/// The body of an [Pet] creation request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreatePetHttpRequestBody {
    pub id: Option<i64>,
    pub name: String,
    pub category: Option<Category>,
    pub photo_urls: Vec<String>,
    pub tags: Option<Vec<Tag>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Error)]
enum ParseCreatePetHttpRequestError {
    #[error(transparent)]
    Name(#[from] PetNameError),
    #[error(transparent)]
    Category(#[from] CategoryError),
    #[error(transparent)]
    PhotoUrls(#[from] PhotoUrlsError),
    #[error(transparent)]
    Tags(#[from] TagsError),
    #[error(transparent)]
    Status(#[from] StatusError),
}

impl CreatePetHttpRequestBody {
    /// Converts the HTTP request body into a domain request.
    fn try_into_domain(self) -> Result<CreatePetRequest, ParseCreatePetHttpRequestError> {
        let name = PetName::new(&self.name)?;
        let category = Category::try_from(self.category)?;
        let photo_urls = PhotoUrls::new(&self.photo_urls)?;
        let tags = Tags::new(&self.tags)?;
        let status = Status::try_from(self.status)?;
        Ok(CreatePetRequest::new(self.id, name.into_inner(), Some(category), photo_urls.into_inner(), tags.into_inner(), Some(status)))
    }
}

/// Create a new [Pet].
///
/// # Responses
///
/// - 201 Created: the [Pet] was successfully created.
/// - 422 Unprocessable entity: An [Pet] with the same name already exists.
pub async fn add_pet<BS: PetService>(
    State(state): State<AppState<BS>>,
    Json(body): Json<CreatePetHttpRequestBody>,
) -> Result<ApiSuccess<CreatePetResponseData>, ApiError> {
    let domain_req = body.try_into_domain()?;
    state
        .pet_service
        .add_pet(&domain_req)
        .await
        .map_err(ApiError::from)
        .map(|ref pet| ApiSuccess::new(StatusCode::CREATED, pet.into()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use axum::http::StatusCode;
    use crate::domain::petstore::models::pet::{Pet, CreatePetRequest, CreatePetError, Status};
    use crate::domain::petstore::models::category::Category;
    use crate::domain::petstore::models::tag::Tag;
    use crate::domain::petstore::ports::PetService;
    use super::*;

    #[derive(Clone)]
    struct MockPetService {
        add_pet_result: Arc<std::sync::Mutex<Option<Result<Pet, CreatePetError>>>>,
    }

    impl PetService for MockPetService {
        async fn add_pet(
            &self,
            _: &CreatePetRequest,
        ) -> Result<Pet, CreatePetError> {
            let mut guard = self.add_pet_result.lock().unwrap();
            guard.take().unwrap_or_else(|| Err(CreatePetError::Unknown(anyhow::anyhow!("Mock add_pet result not set"))))
        }

        async fn find_pet_by_id(
            &self,
            _: i64,
        ) -> Result<Option<Pet>, CreatePetError> {
            Ok(None)
        }
    }

    fn create_mock_pet() -> Pet {
        let mut pet = Pet::new(String::from("doggie"));
        pet.set_category(Category::with_values(1, String::from("Dogs")));
        pet.add_photo(String::from("http://example.com/dog.jpg"));
        pet.add_tag(Tag::with_values(1, String::from("friendly")));
        pet.set_status(Status::Available);
        pet
    }

    #[tokio::test]
    async fn test_add_pet_success() {
        // Arrange
        let pet = create_mock_pet();
        let service = MockPetService {
            add_pet_result: Arc::new(std::sync::Mutex::new(Some(Ok(pet.clone())))),
        };
        
        let state = axum::extract::State(AppState {
            pet_service: Arc::new(service),
        });

        let body = axum::extract::Json(CreatePetHttpRequestBody {
            id: None,
            name: "doggie".to_string(),
            category: Some(Category {
                id: Some(1),
                name: Some("Dogs".to_string()),
            }),
            photo_urls: vec!["http://example.com/dog.jpg".to_string()],
            tags: Some(vec![Tag {
                id: Some(1),
                name: Some("friendly".to_string()),
            }]),
            status: Some("available".to_string()),
        });

        let expected = ApiSuccess::new(
            StatusCode::CREATED,
            CreatePetResponseData {
                id: pet.id,
                name: pet.name,
                category: pet.category.map(|c| Category {
                    id: c.id,
                    name: c.name,
                }),
                photo_urls: pet.photo_urls,
                tags: Some(pet.tags.clone()),
                status: pet.status.map(|s| s.to_string()),
            },
        );

        // Act
        let actual = add_pet(state, body).await;

        // Assert
        assert!(
            actual.is_ok(),
            "expected add_pet to succeed, but got {:?}",
            actual
        );

        let actual = actual.unwrap();
        assert_eq!(
            actual, expected,
            "expected ApiSuccess {:?}, but got {:?}",
            expected, actual
        );
    }

    #[tokio::test]
    async fn test_add_pet_duplicate_error() {
        // Arrange
        let service = MockPetService {
            add_pet_result: Arc::new(std::sync::Mutex::new(Some(Err(CreatePetError::Duplicate {
                name: "doggie".to_string(),
            })))),
        };
        
        let state = axum::extract::State(AppState {
            pet_service: Arc::new(service),
        });

        let body = axum::extract::Json(CreatePetHttpRequestBody {
            id: None,
            name: "doggie".to_string(),
            category: None,
            photo_urls: vec!["http://example.com/test.jpg".to_string()],
            tags: None,
            status: None,
        });

        // Act
        let result = add_pet(state, body).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UnprocessableEntity(_)));
        assert!(error.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_add_pet_unknown_error() {
        // Arrange
        let service = MockPetService {
            add_pet_result: Arc::new(std::sync::Mutex::new(Some(Err(CreatePetError::Unknown(
                anyhow::anyhow!("unexpected error"),
            ))))),
        };
        
        let state = axum::extract::State(AppState {
            pet_service: Arc::new(service),
        });

        let body = axum::extract::Json(CreatePetHttpRequestBody {
            id: None,
            name: "doggie".to_string(),
            category: None,
            photo_urls: vec!["http://example.com/test.jpg".to_string()],
            tags: None,
            status: None,
        });

        // Act
        let result = add_pet(state, body).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::InternalServerError(_)));
    }

    #[tokio::test]
    async fn test_add_pet_invalid_request() {
        // Arrange
        let service = MockPetService {
            add_pet_result: Arc::new(std::sync::Mutex::new(Some(Ok(create_mock_pet())))),
        };
        
        let state = axum::extract::State(AppState {
            pet_service: Arc::new(service),
        });

        let body = axum::extract::Json(CreatePetHttpRequestBody {
            id: None,
            name: "".to_string(),
            category: None,
            photo_urls: vec![],
            tags: None,
            status: Some("invalid_status".to_string()),
        });

        // Act
        let result = add_pet(state, body).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::BadRequest(_)));
    }
}
