use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_gauge, register_histogram,
    Counter, Gauge, Histogram, Encoder, TextEncoder,
};
use std::time::Instant;
use tokio::task;
use warp::Filter;

lazy_static! {
    // 请求计数器
    pub static ref REQUESTS_TOTAL: Counter = register_counter!(
        "nfa_broker_requests_total",
        "Total number of requests"
    ).unwrap();
    
    // 活跃连接数
    pub static ref CONNECTIONS_ACTIVE: Gauge = register_gauge!(
        "nfa_broker_connections_active",
        "Number of active connections"
    ).unwrap();
    
    // 注册服务数
    pub static ref SERVICES_REGISTERED: Gauge = register_gauge!(
        "nfa_broker_services_registered",
        "Number of registered services"
    ).unwrap();
    
    // 请求延迟直方图
    pub static ref REQUEST_DURATION: Histogram = register_histogram!(
        "nfa_broker_request_duration_seconds",
        "Request duration in seconds"
    ).unwrap();
    
    // 错误计数器
    pub static ref ERRORS_TOTAL: Counter = register_counter!(
        "nfa_broker_errors_total",
        "Total number of errors"
    ).unwrap();
    
    // 内存使用量
    pub static ref MEMORY_USAGE: Gauge = register_gauge!(
        "nfa_broker_memory_usage_bytes",
        "Memory usage in bytes"
    ).unwrap();
}

/// 指标中间件
pub fn metrics_middleware() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("metrics").and_then(serve_metrics)
}

/// 提供指标端点
async fn serve_metrics() -> Result<impl warp::Reply, warp::Rejection> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

/// 请求计时器
pub struct RequestTimer {
    start: Instant,
    metric: &'static Histogram,
}

impl RequestTimer {
    pub fn new(metric: &'static Histogram) -> Self {
        Self {
            start: Instant::now(),
            metric,
        }
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metric.observe(duration.as_secs_f64());
    }
}

/// 更新内存使用指标
pub async fn update_memory_metrics() {
    task::spawn_blocking(|| {
        if let Ok(usage) = memory_stats::memory_stats() {
            MEMORY_USAGE.set(usage.physical_mem as f64);
        }
    });
}

/// 初始化指标
pub fn init_metrics() {
    // 注册所有指标
    lazy_static::initialize(&REQUESTS_TOTAL);
    lazy_static::initialize(&CONNECTIONS_ACTIVE);
    lazy_static::initialize(&SERVICES_REGISTERED);
    lazy_static::initialize(&REQUEST_DURATION);
    lazy_static::initialize(&ERRORS_TOTAL);
    lazy_static::initialize(&MEMORY_USAGE);
    
    // 启动定期指标更新
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            update_memory_metrics().await;
        }
    });
}