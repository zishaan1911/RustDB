use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Security audit event
pub struct AuditEvent
{
    pub key_id: String,
    pub action: String,
    pub ip: String,
    pub success: bool,
}

/// Write immutable audit log
pub fn write_audit(event: AuditEvent)
{
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("audit.log")
    {
        let timestamp = now_ns();

        let line = format!(
            "{}|{}|{}|{}|{}\n",
            timestamp,
            event.key_id,
            event.action,
            event.ip,
            event.success
        );

        let _ = file.write_all(line.as_bytes());
    }
}

fn now_ns() -> i64
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}