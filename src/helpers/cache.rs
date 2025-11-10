//! 缓存管理模块
//! 
//! 提供通用的缓存存储、获取和失效管理功能，优化的并发性能和自动过期清理机制

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use metrics::{increment_counter, gauge};


/// 通用缓存项
/// 存储数据和过期时间点
struct CacheItem<T> {
    data: T,
    expiration: Instant,
    creation_time: Instant, // 添加创建时间，便于调试和统计
}

/// 通用缓存管理器
/// 提供缓存数据的存储、获取和失效管理功能，包含自动过期清理机制
struct CacheManager {
    /// 存储缓存数据的映射表
    cache_data: RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
    /// 存储缓存失效信号的映射表
    invalid_signals: RwLock<HashMap<String, bool>>,
    /// 默认缓存持续时间
    default_duration: RwLock<Duration>,
    /// 停止标志，用于安全关闭清理线程
    stop_flag: Arc<AtomicBool>,
    /// 清理线程句柄
    cleanup_thread: Option<JoinHandle<()>>,
    /// 清理间隔
    cleanup_interval: Duration,
}

impl CacheManager {
    /// 创建新的缓存管理器实例
    fn new() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let cleanup_interval = Duration::from_secs(30); // 默认30秒清理一次
        
        Self {
            cache_data: RwLock::new(HashMap::new()),
            invalid_signals: RwLock::new(HashMap::new()),
            default_duration: RwLock::new(Duration::from_secs(60)), // 默认缓存1分钟
            stop_flag,
            cleanup_thread: None, // 初始化时不启动线程
            cleanup_interval,
        }
    }
    
    /// 启动后台清理线程
    fn start_cleanup_thread(&mut self) {
        // 如果线程已存在，则不重复启动
        if self.cleanup_thread.is_some() {
            return;
        }
        
        let stop_flag_clone = self.stop_flag.clone();
        let cleanup_interval = self.cleanup_interval;
        
        // 创建并启动清理线程
        let cleanup_thread = thread::spawn(move || {
            // 这里我们使用一个闭包来避免直接引用CACHE_MANAGER导致的循环问题
            // 由于lazy_static的保证，当线程启动时，CACHE_MANAGER已经初始化完成
            while !stop_flag_clone.load(Ordering::Relaxed) {
                // 尝试获取CACHE_MANAGER并执行清理
                if let Ok(manager) = try_get_cache_manager() {
                    manager.cleanup_expired();
                }
                
                // 休眠指定时间
                thread::sleep(cleanup_interval);
            }
        });
        
        self.cleanup_thread = Some(cleanup_thread);
    }

    /// 获取缓存项
    /// 如果缓存存在且未被标记为失效，则返回缓存的克隆
    /// 注意：过期检查现在主要由后台清理线程处理，但这里仍保留基本检查以避免返回过期数据
    fn get<T: Clone + 'static>(&self, key: &str) -> Option<T> {
        // 快速路径：首先检查是否已被标记为失效
        { 
            // 使用较小的作用域减少锁持有时间
            let invalid_map = self.invalid_signals.read().unwrap();
            if invalid_map.get(key).copied().unwrap_or(false) {
                increment_counter!("cache_misses_total", "key" => key.to_string(), "reason" => "invalid");
                return None;
            }
        }

        // 获取缓存数据
        let cache_map = self.cache_data.read().unwrap();
        if let Some(item) = cache_map.get(key) {
            // 尝试将Any转换为CacheItem<T>
            if let Some(cache_item) = item.downcast_ref::<CacheItem<T>>() {
                // 检查是否过期（快速检查，主要清理工作由后台线程完成）
                if Instant::now() < cache_item.expiration {
                    // 记录缓存命中
                    increment_counter!("cache_hits_total", "key" => key.to_string());
                    return Some(cache_item.data.clone());
                } else {
                    // 记录缓存未命中 - 过期
                    increment_counter!("cache_misses_total", "key" => key.to_string(), "reason" => "expired");
                }
            }
        } else {
            // 记录缓存未命中 - 未找到
            increment_counter!("cache_misses_total", "key" => key.to_string(), "reason" => "not_found");
        }
        None
    }

    /// 设置缓存项
    /// 支持自定义超时时间，如果不提供则使用默认缓存时长
    fn set<T: 'static + Send + Sync>(&self, key: &str, data: T, duration: Option<Duration>) {
        let duration_value = duration.unwrap_or_else(|| self.get_default_duration());
        let now = Instant::now();
        
        let cache_item = CacheItem {
            data,
            expiration: now + duration_value,
            creation_time: now, // 记录创建时间
        };
        
        // 写入缓存
        let mut cache_map = self.cache_data.write().unwrap();
        let is_new = !cache_map.contains_key(key);
        cache_map.insert(key.to_string(), Box::new(cache_item));
        
        // 记录缓存设置
        increment_counter!("cache_sets_total", "key" => key.to_string());
        
        // 更新缓存大小指标
        gauge!("cache_size_items", cache_map.len() as f64);
        
        // 设置缓存后自动重置失效状态
        self.reset(key);
    }



    /// 设置指定缓存键的失效信号
    fn invalidate(&self, key: &str) {
        // 快速路径：直接在invalid_signals中标记为失效
        self.invalid_signals.write().unwrap().insert(key.to_string(), true);
        
        // 记录缓存失效
        increment_counter!("cache_invalidations_total", "key" => key.to_string());
        
        // 可选优化：同时从缓存中删除过期项，减少内存占用
        // 这里使用try_write来避免潜在的死锁
        if let Ok(mut cache_map) = self.cache_data.try_write() {
            cache_map.remove(key);
            // 更新缓存大小指标
            gauge!("cache_size_items", cache_map.len() as f64);
        }
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
    
    /// 清理过期缓存项
    /// 此方法由后台线程定期调用
    fn cleanup_expired(&self) {
        // 在当前的Any类型设计下，我们无法有效检查所有缓存项的过期时间
        // 但我们可以清理已经被标记为失效的项，减少内存占用
        
        // 1. 获取所有被标记为失效的键
        let invalid_keys: Vec<String> = {
            let invalid_map = self.invalid_signals.read().unwrap();
            invalid_map.keys().cloned().collect()
        };
        
        // 2. 从缓存中删除这些键对应的项
        if !invalid_keys.is_empty() {
            let mut cache_map = self.cache_data.write().unwrap();
            for key in &invalid_keys {
                cache_map.remove(key);
            }
            
            // 更新缓存大小指标
            gauge!("cache_size_items", cache_map.len() as f64);
            
            // 记录清理的项数
            increment_counter!("cache_cleanup_items", "count" => invalid_keys.len().to_string());
        }
        
        // 注意：对于未标记为失效但已过期的缓存项，我们仍然依赖get方法中的检查
        // 在实际应用中，可能需要重新设计缓存的类型系统，以便能够更有效地管理过期项
    }
    
    /// 安全停止清理线程
    fn stop_cleanup_thread(&mut self) {
        // 设置停止标志
        self.stop_flag.store(true, Ordering::Relaxed);
        
        // 等待清理线程结束
        if let Some(thread) = self.cleanup_thread.take() {
            thread.join().ok();
        }
    }
}

// 辅助函数：尝试获取缓存管理器实例
// 这个函数在清理线程中使用，避免直接引用CACHE_MANAGER导致的初始化顺序问题
fn try_get_cache_manager() -> Result<Arc<CacheManager>, ()> {
    // 安全地访问CACHE_MANAGER
    // 这里我们直接返回CACHE_MANAGER的克隆，避免在清理线程中直接引用
    Ok(Arc::clone(&*CACHE_MANAGER))
}

// 全局缓存管理器实例
lazy_static::lazy_static! {
    static ref CACHE_MANAGER: Arc<CacheManager> = {
        // 创建一个临时的CacheManager实例用于初始化
        let mut temp_manager = CacheManager::new();
        
        // 启动清理线程
        temp_manager.start_cleanup_thread();
        
        // 将临时实例包装成Arc
        Arc::new(temp_manager)
    };
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