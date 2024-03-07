//! This module provides a simple in-memory key-value storage.
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static! {
    static ref STORAGE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub(crate) async fn get(key: &str) -> Option<String> {
    STORAGE.read().await.get(key).cloned()
}

pub(crate) async fn set(key: String, value: String) {
    STORAGE.write().await.insert(key, value);
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn set_and_get() {
        set("key".to_string(), "value".to_string()).await;
        assert_eq!(get("key").await, Some("value".to_string()));
    }

    #[tokio::test]
    async fn get_absent_key() {
        assert_eq!(get("absent").await, None);
    }
}
