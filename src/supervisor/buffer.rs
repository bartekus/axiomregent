use std::collections::VecDeque;
use std::sync::Mutex;

/// A fixed-size ring buffer for storing log lines.
/// It keeps the last N lines of output.
pub struct LogBuffer {
    capacity: usize,
    buffer: Mutex<VecDeque<String>>,
}

impl LogBuffer {
    /// Create a new LogBuffer with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
        }
    }

    /// Appends a line to the buffer. If the buffer is full, the oldest line is removed.
    pub fn push(&self, line: String) {
        let mut buf = self.buffer.lock().unwrap();
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(line);
    }

    /// Read all lines currently in the buffer.
    pub fn read(&self) -> Vec<String> {
        let buf = self.buffer.lock().unwrap();
        buf.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_capacity() {
        let buffer = LogBuffer::new(3);
        buffer.push("line 1".to_string());
        buffer.push("line 2".to_string());
        buffer.push("line 3".to_string());

        assert_eq!(
            buffer.read(),
            vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string()
            ]
        );

        buffer.push("line 4".to_string());
        assert_eq!(
            buffer.read(),
            vec![
                "line 2".to_string(),
                "line 3".to_string(),
                "line 4".to_string()
            ]
        );
    }

    #[test]
    fn test_log_buffer_empty() {
        let buffer = LogBuffer::new(10);
        assert!(buffer.read().is_empty());
    }
}
