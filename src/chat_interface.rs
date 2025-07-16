use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Padding},
    Frame,
};

/// Represents a single chat message.
#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub content: String,
}

/// Manages the chat interface state and rendering.
pub struct ChatInterface {
    pub messages: Vec<Message>,
    pub input: String,
    pub input_cursor_position: usize,
    pub scroll_offset: usize,
    pub scroll_to_bottom: bool,
}

impl ChatInterface {
    /// Create a new chat interface.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            input_cursor_position: 0,
            scroll_offset: 0,
            scroll_to_bottom: false,
        }
    }

    /// Add a message to the chat history.
    pub fn add_message(&mut self, sender: &str, content: &str) {
        self.messages.push(Message {
            sender: sender.to_string(),
            content: content.to_string(),
        });
        self.scroll_to_bottom = true;
    }

    /// Handle user input (character, backspace, enter, etc).
    pub fn handle_input(&mut self, key: char) {
        match key {
            '\n' => {
                if !self.input.trim().is_empty() {
                    let input = std::mem::take(&mut self.input);
                    self.add_message("User", &input);
                    self.input_cursor_position = 0;
                }
            }
            '\x08' | '\x7f' => {
                // Backspace
                if self.input_cursor_position > 0 {
                    self.input.remove(self.input_cursor_position - 1);
                    self.input_cursor_position -= 1;
                }
            }
            '\x1b' => {
                // Escape key - clear input
                self.input.clear();
                self.input_cursor_position = 0;
            }
            c if c.is_ascii() && !c.is_control() => {
                if self.input_cursor_position > self.input.len() {
                    self.input_cursor_position = self.input.len();
                }
                self.input.insert(self.input_cursor_position, c);
                self.input_cursor_position += 1;
            }
            _ => {}
        }
        // Clamp cursor position to input length
        if self.input_cursor_position > self.input.len() {
            self.input_cursor_position = self.input.len();
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Render the chat interface (conversation + input area).
    pub fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3), // Conversation history
                Constraint::Length(3), // Input area
            ])
            .split(frame.area());

        self.render_conversation_history(frame, chunks[0]);
        self.render_input_area(frame, chunks[1]);
    }

    /// Render the conversation history area.
    fn render_conversation_history(&mut self, frame: &mut Frame, area: Rect) {
        let mut conversation_text = Vec::new();
        for msg in &self.messages {

            let sender_style = if msg.sender == "User" {
                Style::default().fg(Color::Rgb(0x4D, 0xC8, 0xCD)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(0xF4, 0xC5, 0x4C)).add_modifier(Modifier::BOLD)
            };

            let content_style = Style::default().fg(Color::White);
            // Add sender line
            conversation_text.push(Line::from(vec![
                Span::styled(format!("{}: ", msg.sender), sender_style),
            ]));

            // Split content into lines that fit the width
            let max_width = area.width.saturating_sub(4) as usize; // Account for borders
            let words: Vec<&str> = msg.content.split_whitespace().collect();
            let mut current_line = String::new();
            for word in words {
                if word.len() > max_width {
                    // Break the word into chunks of max_width
                    for chunk in word.as_bytes().chunks(max_width) {
                        if !current_line.is_empty() {
                            conversation_text.push(Line::from(vec![
                                Span::styled(current_line.clone(), content_style),
                            ]));
                            current_line.clear();
                        }
                        let chunk_str = String::from_utf8_lossy(chunk).to_string();
                        conversation_text.push(Line::from(vec![
                            Span::styled(chunk_str, content_style),
                        ]));
                    }
                } else if current_line.len() + word.len() < max_width {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                } else {
                    // Start a new line
                    if !current_line.is_empty() {
                        conversation_text.push(Line::from(vec![
                            Span::styled(current_line.clone(), content_style),
                        ]));
                        current_line.clear();
                    }
                    current_line.push_str(word);
                }
            }
            // Add the last line if not empty
            if !current_line.is_empty() {
                conversation_text.push(Line::from(vec![
                    Span::styled(current_line, content_style),
                ]));
            }
            // Add a blank line between messages
            conversation_text.push(Line::from(""));
        }

        // Apply scroll offset
        let available_height = area.height.saturating_sub(2) as usize; // Account for borders
        let total_lines = conversation_text.len();

        // Only scroll to bottom if requested
        if self.scroll_to_bottom {
            if total_lines > available_height {
                self.scroll_offset = total_lines - available_height;
            } else {
                self.scroll_offset = 0;
            }
            self.scroll_to_bottom = false;
        }
        if self.scroll_offset >= total_lines {
            self.scroll_offset = total_lines.saturating_sub(1);
        }
        let start_index = if self.scroll_offset >= total_lines {
            0
        } else {
            self.scroll_offset
        };

        let end_index = (start_index + available_height).min(total_lines);

        let visible_text: Vec<Line> = conversation_text
            .into_iter()
            .skip(start_index)
            .take(end_index - start_index)
            .collect();

        let text = Text::from(visible_text);

        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .title(" Fisher [\"esc\" to quit] ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(0xFD, 0x5F, 0x54)))
                .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(Color::Rgb(0x0D, 0x0C, 0x11)));

        frame.render_widget(paragraph, area);
    }

    /// Render the input area.
    fn render_input_area(&self, frame: &mut Frame, area: Rect) {
        let input_text = self.input.clone();

        let input_style = Style::default()
            .fg(Color::Rgb(0xFF, 0xD6, 0x00))
            .bg(Color::Rgb(0x44, 0x17, 0x1C));

        let paragraph = Paragraph::new(input_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(0xFD, 0x5F, 0x54)))
                .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
            )
            .style(input_style);

        frame.render_widget(paragraph, area);

        // Render cursor after the character at input_cursor_position
        let cursor_x = area.x + 2  + self.input_cursor_position as u16;
        let cursor_y = area.y + 1;

        frame.set_cursor_position((cursor_x, cursor_y));
    }

    /// Get the last message in the chat history.
    pub fn get_last_message(&self) -> Option<&Message> {
        self.messages.last()
    }
}

impl Default for ChatInterface {
    fn default() -> Self {
        Self::new()
    }
} 