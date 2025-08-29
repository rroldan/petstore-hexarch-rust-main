use crate::domain::petstore::models::category::Category;
use crate::domain::petstore::models::tag::Tag;
use crate::domain::petstore::models::pet::Status;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetName(String);

#[derive(Debug, Clone, Error)]
pub enum PetNameError {
    #[error("pet name cannot be empty")]
    Empty,
}

impl PetName {
    pub fn new(name: &str) -> Result<Self, PetNameError> {
        if name.trim().is_empty() {
            return Err(PetNameError::Empty);
        }
        Ok(Self(name.to_string()))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for PetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoUrls(Vec<String>);

#[derive(Debug, Clone, Error)]
pub enum PhotoUrlsError {
    #[error("photo urls cannot be empty")]
    Empty,
}

impl PhotoUrls {
    pub fn new(urls: &[String]) -> Result<Self, PhotoUrlsError> {
        if urls.is_empty() {
            return Err(PhotoUrlsError::Empty);
        }
        Ok(Self(urls.to_vec()))
    }

    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl std::fmt::Display for PhotoUrls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tags(Vec<Tag>);

#[derive(Debug, Clone, Error)]
pub enum TagsError {
    #[error("tags cannot be empty")]
    Empty,
}

impl Tags {
    pub fn new(tags: &Option<Vec<Tag>>) -> Result<Self, TagsError> {
        match tags {
            Some(t) if t.is_empty() => Err(TagsError::Empty),
            Some(t) => Ok(Self(t.clone())),
            None => Ok(Self(vec![])),
        }
    }

    pub fn into_inner(self) -> Vec<Tag> {
        self.0
    }
}

impl std::fmt::Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone, Error)]
pub enum StatusError {
    #[error("invalid status: {invalid_status}")]
    InvalidStatus { invalid_status: String },
}

impl TryFrom<Option<String>> for Status {
    type Error = StatusError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value.as_deref() {
            Some("available") => Ok(Status::Available),
            Some("pending") => Ok(Status::Pending),
            Some("sold") => Ok(Status::Sold),
            Some(s) => Err(StatusError::InvalidStatus { invalid_status: s.to_string() }),
            None => Ok(Status::Available), // Default status
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum CategoryError {
    #[error("invalid category")]
    InvalidCategory,
}

impl TryFrom<Option<Category>> for Category {
    type Error = CategoryError;

    fn try_from(value: Option<Category>) -> Result<Self, Self::Error> {
        match value {
            Some(c) => Ok(c),
            None => Ok(Category::new()),
        }
    }
} 