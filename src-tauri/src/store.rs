use crate::json_store::JsonStore;
use std::sync::Arc;

/// 全局应用状态
pub struct AppState {
    pub db: Arc<JsonStore>,
}

impl AppState {
    pub fn new(db: Arc<JsonStore>) -> Self {
        Self { db }
    }
}
