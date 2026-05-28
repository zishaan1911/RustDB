use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

// Token bucket state per API key
#[derive(Debug, Clone)]
pub struct RateBucket
{
    pub tokens: u32,
    pub last_refill: i64,
}

// Global in-memory limiter (replace with Redis in prod cluster)
lazy_static::lazy_static!
{
    static ref BUCKETS: Mutex<HashMap<String, RateBucket>> =
        Mutex::new(HashMap::new());
}

/// Config
const MAX_TOKENS: u32 = 100;
const REFILL_PER_SEC: u32 = 10;

/// Check rate limit
pub fn check_rate_limit(key_id: &str) -> bool
{
    let mut buckets = BUCKETS.lock().unwrap();

    let now = now_ns();

    let bucket = buckets.entry(key_id.to_string()).or_insert(
        RateBucket {
            tokens: MAX_TOKENS,
            last_refill: now,
        }
    );

    let elapsed = (now - bucket.last_refill) / 1_000_000_000;

    if elapsed > 0
    {
        let refill = (elapsed as u32) * REFILL_PER_SEC;
        bucket.tokens = (bucket.tokens + refill).min(MAX_TOKENS);
        bucket.last_refill = now;
    }

    if bucket.tokens == 0
    {
        return false;
    }

    bucket.tokens -= 1;
    true
}

fn now_ns() -> i64
{
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}