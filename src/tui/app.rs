use std::collections::VecDeque;

pub struct App {
    status_messages: VecDeque<String>,
    max_messages: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            status_messages: VecDeque::with_capacity(100),
            max_messages: 100,
        }
    }

    pub fn add_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_messages.push_back(message.into());
        if self.status_messages.len() > self.max_messages {
            self.status_messages.pop_front();
        }
    }

    pub fn status_messages(&self) -> &VecDeque<String> {
        &self.status_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app() {
        let mut app = App::new();

        // Test adding messages
        app.add_status_message("Test message 1");
        app.add_status_message("Test message 2");
        assert_eq!(app.status_messages().len(), 2);

        // Test message limit
        for i in 0..200 {
            app.add_status_message(format!("Message {}", i));
        }
        assert_eq!(app.status_messages().len(), 100);
    }
}
