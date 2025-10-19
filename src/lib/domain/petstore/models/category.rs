use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Category {
    pub id: Option<i64>,
    pub name: Option<String>
}

impl Default for Category {
    fn default() -> Self {
        Self::new()
    }
}

impl Category {
    /// A category for a pet
    pub fn new() -> Category {
        Category {
            id: None,
            name: None,
        }
    }

    pub fn with_values(id: i64, name: String) -> Self {
        Category {
            id: Some(id),
            name: Some(name),
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Category(id: {:?}, name: {:?})", self.id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_category() {
        let category = Category::new();
        assert!(category.id.is_none());
        assert!(category.name.is_none());
    }

    #[test]
    fn test_category_with_values() {
        let id = 1;
        let name = String::from("Dogs");
        let category = Category::with_values(id, name.clone());
        
        assert_eq!(category.id, Some(id));
        assert_eq!(category.name, Some(name));
    }

    #[test]
    fn test_category_clone() {
        let category = Category::with_values(1, String::from("Cats"));
        let cloned = category.clone();
        
        assert_eq!(category, cloned);
    }

    #[test]
    fn test_category_ordering() {
        let cat1 = Category::with_values(1, String::from("Dogs"));
        let cat2 = Category::with_values(2, String::from("Cats"));
        
        assert!(cat1 < cat2); // Orders by id first
    }

    #[test]
    fn test_category_manual_update() {
        let mut category = Category::new();
        
        assert!(category.id.is_none());
        category.id = Some(5);
        assert_eq!(category.id, Some(5));
        
        assert!(category.name.is_none());
        category.name = Some(String::from("Birds"));
        assert_eq!(category.name, Some(String::from("Birds")));
    }
}