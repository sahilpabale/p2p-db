use futures_locks::RwLock;
use std::collections::HashMap;

pub struct KVStore {
    pub store: RwLock<HashMap<String, String>>,
}

impl KVStore {
    pub fn new() -> Self {
        KVStore {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut store = self.store.write().await;
        store.insert(key, value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        store.get(key).cloned()
    }
}
