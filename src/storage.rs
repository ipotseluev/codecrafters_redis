//! This module provides a simple in-memory key-value storage.
use crate::protocol::Set;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static! {
    static ref STORAGE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

/// Wrapper around the in-memory storage.
pub struct Storage {
    inner: &'static RwLock<HashMap<String, String>>,
}

impl Storage {
    pub fn instance() -> Self {
        Self { inner: &STORAGE }
    }

    /// Get value from storage by key.
    pub async fn get(&self, key: &str) -> Option<String> {
        self.inner.read().await.get(key).cloned()
    }

    /// Store key-value pair in the storage.
    pub async fn set(&self, request: Set) {
        self.inner.write().await.insert(request.key, request.value);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn set_and_get() {
        let storage = Storage::instance();
        let request = Set {
            key: "key".to_string(),
            value: "value".to_string(),
            expiration_timeout_ms: None,
        };
        storage.set(request).await;
        assert_eq!(storage.get("key").await, Some("value".to_string()));
    }

    #[tokio::test]
    async fn get_absent_key() {
        let storage = Storage::instance();
        assert_eq!(storage.get("absent").await, None);
    }
}
