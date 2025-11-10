//! 缓存管理模块
//! 
//! 提供通用的缓存存储、获取和失效管理功能

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};


/// 通用缓存项
/// 存储数据和过期时间点
struct CacheItem<T> {
    data: T,
    expiration: Instant,
}

/// 通用缓存管理器
/// 提供缓存数据的存储、获取和失效管理功能
struct CacheManager {
    /// 存储缓存数据的映射表
    cache_data: RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
    /// 存储缓存失效信号的映射表
    invalid_signals: RwLock<HashMap<String, bool>>,
    /// 默认缓存持续时间
    default_duration: RwLock<Duration>,
}

impl CacheManager {
    /// 创建新的缓存管理器实例
    fn new() -> Self {
        Self {
            cache_data: RwLock::new(HashMap::new()),
            invalid_signals: RwLock::new(HashMap::new()),
            default_duration: RwLock::new(Duration::from_secs(60)), // 默认缓存1分钟
        }
    }

    /// 获取缓存项
    /// 如果缓存存在且未过期且未被标记为失效，则返回缓存的克隆
    fn get<T: Clone + 'static>(&self, key: &str) -> Option<T> {
        // 检查是否已被标记为失效
        if self.is_invalid(key) {
            return None;
        }

        // 获取缓存数据
        let cache_map = self.cache_data.read().unwrap();
        if let Some(item) = cache_map.get(key) {
            // 尝试将Any转换为CacheItem<T>
            if let Some(cache_item) = item.downcast_ref::<CacheItem<T>>() {
                // 检查是否过期
                if Instant::now() < cache_item.expiration {
                    return Some(cache_item.data.clone());
                }
            }
        }
        None
    }

    /// 设置缓存项
    /// 支持自定义超时时间，如果不提供则使用默认缓存时长
    fn set<T: 'static + Send + Sync>(&self, key: &str, data: T, duration: Option<Duration>) {
        let duration_value = duration.unwrap_or_else(|| self.get_default_duration());
        let cache_item = CacheItem {
            data,
            expiration: Instant::now() + duration_value,
        };
        self.cache_data.write().unwrap().insert(key.to_string(), Box::new(cache_item));
        // 设置缓存后自动重置失效状态
        self.reset(key);
    }



    /// 设置指定缓存键的失效信号
    fn invalidate(&self, key: &str) {
        self.invalid_signals.write().unwrap().insert(key.to_string(), true);
    }

    /// 检查指定缓存键是否已被标记为失效
    fn is_invalid(&self, key: &str) -> bool {
        self.invalid_signals.read().unwrap().get(key).copied().unwrap_or(false)
    }

    /// 重置指定缓存键的失效状态
    fn reset(&self, key: &str) {
        self.invalid_signals.write().unwrap().remove(key);
    }



    /// 获取默认缓存持续时间
    fn get_default_duration(&self) -> Duration {
        *self.default_duration.read().unwrap()
    }


}

// 全局缓存管理器实例
lazy_static::lazy_static! {
    static ref CACHE_MANAGER: Arc<CacheManager> = Arc::new(CacheManager::new());
}

/// 使指定缓存键失效
/// 
/// # 参数
/// - `key`: 缓存键，由调用方提供的字符串标识
/// 
/// # 示例
/// ```
/// // 使待办事项缓存失效
/// invalidate_cache("todos");
/// 
/// // 使用户缓存失效
/// invalidate_cache("users");
/// ```
pub fn invalidate_cache(key: &str) {
    CACHE_MANAGER.invalidate(key);
}



/// 从缓存获取数据
/// 
/// # 参数
/// - `key`: 缓存键，由调用方提供的字符串标识
/// 
/// # 返回值
/// - 如果缓存存在且未过期且未被标记为失效，则返回缓存的克隆，否则返回 `None`
/// 
/// # 示例
/// ```
/// if let Some(todos) = get_from_cache("todos") {
///     // 使用缓存的待办事项
/// }
/// ```
pub fn get_from_cache<T: Clone + 'static>(key: &str) -> Option<T> {
    CACHE_MANAGER.get(key)
}

/// 向缓存中设置数据
/// 
/// # 参数
/// - `key`: 缓存键，由调用方提供的字符串标识
/// - `data`: 要缓存的数据
/// - `duration`: 可选的缓存持续时间，如果不提供则使用默认值
/// 
/// # 示例
/// ```
/// // 使用默认缓存时间缓存待办事项
/// set_to_cache("todos", todo_list, None);
/// 
/// // 使用自定义缓存时间（30秒）
/// use std::time::Duration;
/// set_to_cache("todos", todo_list, Some(Duration::from_secs(30)));
/// ```
pub fn set_to_cache<T: 'static + Send + Sync>(key: &str, data: T, duration: Option<Duration>) {
    CACHE_MANAGER.set(key, data, duration);
}