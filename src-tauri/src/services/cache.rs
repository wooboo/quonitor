use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::providers::QuotaData;

#[derive(Clone)]
pub struct Cache {
    data: Arc<RwLock<HashMap<String, QuotaData>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set(&self, account_id: String, quota: QuotaData) {
        let mut data = self.data.write().await;
        data.insert(account_id, quota);
    }

    pub async fn get(&self, account_id: &str) -> Option<QuotaData> {
        let data = self.data.read().await;
        data.get(account_id).cloned()
    }

    pub async fn get_all(&self) -> Vec<QuotaData> {
        let data = self.data.read().await;
        data.values().cloned().collect()
    }

    pub async fn remove(&self, account_id: &str) {
        let mut data = self.data.write().await;
        data.remove(account_id);
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }
}
