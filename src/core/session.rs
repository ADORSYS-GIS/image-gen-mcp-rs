use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Stores image data/URLs associated with unique IDs for the duration of a session.
#[derive(Debug, Clone, Default)]
pub struct SessionStore {
    images: Arc<RwLock<HashMap<String, String>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn store(&self, id: String, data: String) {
        let mut images = self.images.write().await;
        images.insert(id, data);
    }

    pub async fn get(&self, id: &str) -> Option<String> {
        let images = self.images.read().await;
        images.get(id).cloned()
    }
}
