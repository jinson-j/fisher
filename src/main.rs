use std::io;
use dotenv::dotenv;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

mod ui;
use ui::chat_interface;
use ui::chat_interface::ChatInterface;
use ui::home_screen::{HomeScreen, HomeScreenAction};

mod model;
use model::{
    generate_response,
};

mod files;
use files::setup_vector_store;

mod faiss;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    if let Err(e) = run_app().await {
        eprintln!("App error: {}", e);
    }
}

/// Main application loop with home screen and chat
async fn run_app() -> Result<(), io::Error> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut home_screen = HomeScreen::new();
    let mut current_directory = home_screen.get_directory();

    // Home screen loop
    loop {
        terminal.draw(|f| {
            home_screen.render(f);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(c) => {
                        let action = home_screen.handle_input(c);
                        match action {
                            HomeScreenAction::StartChat => {
                                current_directory = home_screen.get_directory();
                                break;
                            }
                            HomeScreenAction::Quit => {
                                // Restore terminal and exit
                                disable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    LeaveAlternateScreen,
                                    DisableMouseCapture
                                )?;
                                terminal.show_cursor()?;
                                return Ok(());
                            }
                            HomeScreenAction::Continue => {}
                        }
                    }
                    KeyCode::Backspace => {
                        let action = home_screen.handle_input('\x08');
                        if action == HomeScreenAction::Quit {
                            // Restore terminal and exit
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
                            return Ok(());
                        }
                    }
                    KeyCode::Enter => {
                        let action = home_screen.handle_input('\n');
                        if action == HomeScreenAction::StartChat {
                            current_directory = home_screen.get_directory();
                            break;
                        }
                    }
                    KeyCode::Esc => {
                        // Restore terminal and exit
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }


    // set up the vector store
    setup_vector_store(current_directory);

    return Ok(());

    // Chat loop
    let mut chat = ChatInterface::new();

    loop {
        let last_message = chat.get_last_message();
        if let Some(message) = last_message {
            if message.sender == "User" {
                chat.add_message("LLM", "...");
                terminal.draw(|f| {
                    chat.render(f);
                })?;
                // Handle LLM response
                match generate_response(&chat.messages).await {
                    Ok(response) => {
                        chat.messages.pop(); // Remove waiting message
                        chat.add_message("LLM", &response);
                    }
                    Err(e) => {
                        chat.messages.pop();
                        chat.add_message("LLM", &format!("Error: {}", e));
                    }
                }
            }
        }

        terminal.draw(|f| {
            chat.render(f);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(c) => chat.handle_input(c),
                    KeyCode::Backspace => chat.handle_input('\x08'),
                    KeyCode::Enter => chat.handle_input('\n'),
                    KeyCode::Up => chat.scroll_up(),
                    KeyCode::Down => chat.scroll_down(),
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}