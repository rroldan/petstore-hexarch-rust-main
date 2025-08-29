use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: Option<String>,
}

impl Tag {
    pub fn new() -> Tag {
        Tag {
            id: None,
            name: None,
        }
    }

    pub fn with_values(id: i64, name: String) -> Self {
        Tag {
            id: Some(id),
            name: Some(name),
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag(id: {:?}, name: {:?})", self.id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tag() {
        let tag = Tag::new();
        assert!(tag.id.is_none());
        assert!(tag.name.is_none());
    }

    #[test]
    fn test_tag_with_values() {
        let id = 1;
        let name = String::from("Friendly");
        let tag = Tag::with_values(id, name.clone());
        
        assert_eq!(tag.id, Some(id));
        assert_eq!(tag.name, Some(name));
    }

    #[test]
    fn test_tag_clone() {
        let tag = Tag::with_values(1, String::from("Playful"));
        let cloned = tag.clone();
        
        assert_eq!(tag, cloned);
    }

    #[test]
    fn test_tag_ordering() {
        let tag1 = Tag::with_values(1, String::from("Friendly"));
        let tag2 = Tag::with_values(2, String::from("Active"));
        
        assert!(tag1 < tag2); // Orders by id first
    }

    #[test]
    fn test_tag_manual_update() {
        let mut tag = Tag::new();
        
        assert!(tag.id.is_none());
        tag.id = Some(5);
        assert_eq!(tag.id, Some(5));
        
        assert!(tag.name.is_none());
        tag.name = Some(String::from("Quiet"));
        assert_eq!(tag.name, Some(String::from("Quiet")));
    }
}