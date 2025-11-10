//! 安全相关功能模块
//! 
//! 提供日志敏感信息清理功能

/// 清理日志消息，移除敏感信息
/// 
/// 此函数用于处理日志文本，检测并替换常见的敏感信息（如密码、令牌等）
/// 
/// # 参数
/// - `message`: 原始日志消息文本
/// 
/// # 返回值
/// 返回清理后的日志消息，敏感信息被替换为星号
pub fn sanitize_log_message(message: &str) -> String {
    // 简单的敏感信息检测和替换逻辑
    let mut sanitized = message.to_string();
    
    // 检测并替换密码相关的敏感信息
    if let Some(index) = sanitized.find("password=") {
        let start = index + 9; // "password=" 的长度
        let end = sanitized[start..].find(&['&', ' ', '\n', '\r', '\t'][..]).unwrap_or_else(|| sanitized.len() - start) + start;
        
        if end > start {
            sanitized.replace_range(start..end, "********");
        }
    }
    
    // 检测并替换API密钥相关的敏感信息
    if let Some(index) = sanitized.find("api_key=") {
        let start = index + 8; // "api_key=" 的长度
        let end = sanitized[start..].find(&['&', ' ', '\n', '\r', '\t'][..]).unwrap_or_else(|| sanitized.len() - start) + start;
        
        if end > start {
            sanitized.replace_range(start..end, "********");
        }
    }
    
    // 检测并替换令牌相关的敏感信息
    if let Some(index) = sanitized.find("token=") {
        let start = index + 6; // "token=" 的长度
        let end = sanitized[start..].find(&['&', ' ', '\n', '\r', '\t'][..]).unwrap_or_else(|| sanitized.len() - start) + start;
        
        if end > start {
            sanitized.replace_range(start..end, "********");
        }
    }
    
    // 检测并替换JWT令牌
    if let Some(index) = sanitized.find("Bearer ") {
        let start = index + 7; // "Bearer " 的长度
        let end = sanitized[start..].find(&[' ', '\n', '\r', '\t'][..]).unwrap_or_else(|| sanitized.len() - start) + start;
        
        if end > start && sanitized[start..end].contains('.') {
            // 简单的JWT检测，真正的JWT包含点号
            sanitized.replace_range(start..end, "********");
        }
    }
    
    sanitized
}