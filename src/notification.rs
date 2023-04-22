#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub ttl: u16,
}

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Error,
    Warning,
    Info,
}

impl Notification {
    pub fn new(message: String, level: NotificationLevel) -> Self {
        Self {
            message,
            level,
            ttl: 8,
        }
    }
}
