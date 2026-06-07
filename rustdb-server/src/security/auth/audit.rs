use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! 
{
    static ref AUDIT_LOG: Mutex<Option<File>> = Mutex::new(open_audit_log());
}

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
    let mut file = match AUDIT_LOG.lock() 
    {
        Ok(file) => file,
        Err(err) => {
            eprintln!("failed to lock audit log: {}", err);
            return;
        }
    };

    let Some(file) = file.as_mut() else 
    {
        return;
    };

    let timestamp = now_ns();

    let line = format!(
        "{}|{}|{}|{}|{}\n",
        timestamp, event.key_id, event.action, event.ip, event.success
    );

    if let Err(err) = file.write_all(line.as_bytes()) 
    {
        eprintln!("failed to write audit log: {}", err);
    }
}

fn open_audit_log() -> Option<File> 
{
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open("audit.log")
    {
        Ok(file) => Some(file),
        Err(err) => {
            eprintln!("failed to open audit log: {}", err);
            None
        }
    }
}

fn now_ns() -> i64 
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}
