use serde::{Deserialize, Serialize};

/// 统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub user_count: u64,
    pub project_count: u64,
    pub satisfaction: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            user_count: 1000,
            project_count: 500,
            satisfaction: 98,
        }
    }
}
