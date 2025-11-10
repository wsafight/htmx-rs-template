use serde::{Deserialize, Serialize};

/// 分页查询参数结构体
#[derive(Debug, Deserialize)]
pub struct PageQuery {
    /// 页码，默认为1
    pub page: Option<i64>,
    /// 每页数量，默认为12，范围1-100
    pub per_page: Option<i64>,
}

/// 分页信息结构体
#[derive(Clone, Debug, Serialize)]
pub struct Pagination {
    /// 当前页码
    pub current_page: i64,
    /// 每页数量
    pub per_page: i64,
    /// 总记录数
    pub total: i64,
    /// 总页数
    pub total_pages: i64,
    /// 是否有上一页
    pub has_prev: bool,
    /// 是否有下一页
    pub has_next: bool,
}

impl PageQuery {
    // 若用户传入的 page ≤ 0，则统一视为第 1 页，避免非法页码导致计算错误
    pub fn get_page(&self) -> i64 {
        self.page.filter(|&p| p > 0).unwrap_or(1)
    }

    /// 获取处理后的每页数量，确保在合理范围内
    pub fn get_per_page(&self) -> i64 {
        self.per_page.unwrap_or(12).clamp(1, 100)
    }

    /// 计算偏移量
    pub fn get_offset(&self) -> i64 {
        (self.get_page() - 1) * self.get_per_page()
    }
}

/// 创建分页信息
///
/// # 参数
/// * `page` - 当前页码
/// * `per_page` - 每页数量
/// * `total` - 总记录数
///
/// # 返回值
/// 构建好的Pagination结构体
pub fn create_pagination(page: i64, per_page: i64, total: i64) -> Pagination {
    let total_pages = if per_page == 0 {
        0
    } else {
        (total as f64 / per_page as f64).ceil() as i64
    };

    Pagination {
        current_page: page,
        per_page,
        total,
        total_pages,
        has_prev: page > 1,
        has_next: page < total_pages,
    }
}

/// 计算显示范围
///
/// # 参数
/// * `page` - 当前页码
/// * `per_page` - 每页数量
/// * `current_count` - 当前页实际记录数
///
/// # 返回值
/// (start_item, end_item) - 开始和结束的项目索引
pub fn calculate_display_range(page: i64, per_page: i64, current_count: usize) -> (i64, i64) {
    let start_item = (page - 1) * per_page + 1;
    let end_item = start_item - 1 + current_count as i64;

    (start_item, end_item)
}
