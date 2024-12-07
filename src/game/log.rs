use crate::engine::Color;

pub struct LogEntry {
    pub string: String,
    pub color: Color,
}

impl LogEntry {

    pub fn new(string: String, color: Color) -> LogEntry {
        LogEntry { string, color }
    }

}