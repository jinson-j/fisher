use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Padding},
    Frame,
};
use std::path::PathBuf;

/// Manages the home screen state and rendering.
pub struct HomeScreen {
    pub directory: String,
    pub directory_cursor_position: usize,
    pub is_editing_directory: bool,
}

impl HomeScreen {
    /// Create a new home screen.
    pub fn new() -> Self {
        Self {
            directory: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            directory_cursor_position: 0,
            is_editing_directory: false,
        }
    }

    /// Handle user input for the home screen.
    pub fn handle_input(&mut self, key: char) -> HomeScreenAction {
        if self.is_editing_directory {
            match key {
                '\n' => {
                    self.is_editing_directory = false;
                    self.directory_cursor_position = 0;
                    HomeScreenAction::Continue
                }
                '\x08' | '\x7f' => {
                    // Backspace
                    if self.directory_cursor_position > 0 {
                        self.directory.remove(self.directory_cursor_position - 1);
                        self.directory_cursor_position -= 1;
                    }
                    HomeScreenAction::Continue
                }
                '\x1b' => {
                    // Escape key - cancel editing
                    self.is_editing_directory = false;
                    self.directory_cursor_position = 0;
                    HomeScreenAction::Continue
                }
                c if c.is_ascii() && !c.is_control() => {
                    if self.directory_cursor_position > self.directory.len() {
                        self.directory_cursor_position = self.directory.len();
                    }
                    self.directory.insert(self.directory_cursor_position, c);
                    self.directory_cursor_position += 1;
                    HomeScreenAction::Continue
                }
                _ => HomeScreenAction::Continue,
            }
        } else {
            match key {
                'd' | 'D' => {
                    self.is_editing_directory = true;
                    self.directory_cursor_position = self.directory.len();
                    HomeScreenAction::Continue
                }
                'c' | 'C' => HomeScreenAction::StartChat,
                '\x1b' => HomeScreenAction::Quit,
                _ => HomeScreenAction::Continue,
            }
        }
    }

    /// Get the current directory as PathBuf.
    pub fn get_directory(&self) -> PathBuf {
        PathBuf::from(&self.directory)
    }

    /// Render the home screen.
    pub fn render(&mut self, frame: &mut Frame) {
        // Horizontal layout for side padding
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4), // Left padding
                Constraint::Min(0),   // Main content
                Constraint::Length(4), // Right padding
            ])
            .split(frame.area());

        // Render left and right padding
        self.render_padding(frame, h_chunks[0]);
        self.render_padding(frame, h_chunks[2]);

        // Vertical layout for main content
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),  // Top padding
                Constraint::Length(8), // ASCII art area
                Constraint::Length(1),  // Title area
                Constraint::Length(3),  // Directory input area (shorter)
                Constraint::Length(1),  // Bottom padding
                Constraint::Length(3),  // Instructions area
                Constraint::Min(1),  // Bottom padding
            ])
            .split(h_chunks[1]);

        self.render_padding(frame, v_chunks[0]);
        self.render_ascii_art(frame, v_chunks[1]);
        self.render_title(frame, v_chunks[2]);
        self.render_directory_input(frame, v_chunks[3]);
        self.render_padding(frame, v_chunks[4]);
        self.render_instructions(frame, v_chunks[5]);
        self.render_padding(frame, v_chunks[6]);
    }

    /// Render the ASCII art.
    fn render_ascii_art(&self, frame: &mut Frame, area: Rect) {
        let ascii_art = vec![
            "          ,-.               ",
            "      O  /   `.             ",
            "     <\\/      `.           ",
            "       |*        `.         ",
            "     / \\          `.       ",
            "     /  /            `>')3s,",
            "-----------               ,'",
            "                         7  ",
        ];

        let mut lines = Vec::new();
        for line in ascii_art {
            lines.push(Line::from(vec![
                Span::styled(line, Style::default().fg(Color::Rgb(0xFD, 0x5F, 0x54))),
            ]));
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().bg(Color::Rgb(0x0D, 0x0C, 0x11)));

        frame.render_widget(paragraph, area);
    }

    /// Render the title.
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title_style = Style::default()
            .fg(Color::Rgb(0xFD, 0x5F, 0x54))
            .add_modifier(Modifier::BOLD);
        
        let title = Line::from(vec![
            Span::styled("Fisher", title_style),
        ]);

        let text = Text::from(title);
        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().bg(Color::Rgb(0x0D, 0x0C, 0x11)));

        frame.render_widget(paragraph, area);
    }

    /// Render the directory input area.
    fn render_directory_input(&self, frame: &mut Frame, area: Rect) {
        let directory_style = if self.is_editing_directory {
            Style::default()
                .fg(Color::Rgb(0xFD, 0x5F, 0x54))
                .bg(Color::Rgb(0x44, 0x17, 0x1C))
                
        } else {
            Style::default()
                .fg(Color::Rgb(0xFD, 0x5F, 0x54))
                .bg(Color::Rgb(0x0D, 0x0C, 0x11))
        };

        let border_style = Style::default().fg(Color::Rgb(0xFD, 0x5F, 0x54));

        let paragraph = Paragraph::new(self.directory.clone())
            .block(Block::default()
                .title(" Directory ")
                .borders(Borders::ALL)
                .border_style(border_style)
                .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
            )
            .style(directory_style);

        frame.render_widget(paragraph, area);

        // Render cursor if editing
        if self.is_editing_directory {
            let cursor_x = area.x + 2 + self.directory_cursor_position as u16;
            let cursor_y = area.y + 1;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    /// Render padding areas with dark background.
    fn render_padding(&self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new("")
            .style(Style::default().bg(Color::Rgb(0x0D, 0x0C, 0x11)));
        frame.render_widget(paragraph, area);
    }

    /// Render the instructions.
    fn render_instructions(&self, frame: &mut Frame, area: Rect) {
        let instructions = if self.is_editing_directory {
            vec![
                "'enter' to confirm directory",
                "'esc' to quit",
            ]
        } else {
            vec![
                "'d' to edit directory",
                "'c' to start chat",
                "'esc' to quit",
            ]
        };


        let mut lines = Vec::new();
        for instruction in instructions {
            lines.push(Line::from(vec![
                Span::styled(instruction, Style::default().fg(Color::Rgb(0xFD, 0x5F, 0x54))),
            ]));
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().bg(Color::Rgb(0x0D, 0x0C, 0x11)));

        frame.render_widget(paragraph, area);
    }
}

/// Actions that can be returned from the home screen.
#[derive(Debug, Clone, PartialEq)]
pub enum HomeScreenAction {
    Continue,
    StartChat,
    Quit,
}

impl Default for HomeScreen {
    fn default() -> Self {
        Self::new()
    }
} 