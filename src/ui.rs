use std::io::{self, Write};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;

use crate::config::Config;
use crate::error::ShellResult;

/// Terminal UI manager
pub struct UiManager {
    config: Config,
}

impl UiManager {
    /// Create a new UI manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Display the shell prompt
    pub fn display_prompt(&self) -> ShellResult<()> {
        if self.config.ui.enable_colors {
            self.display_colored_prompt()?;
        } else {
            self.display_plain_prompt()?;
        }
        Ok(())
    }

    /// Display colored prompt
    fn display_colored_prompt(&self) -> ShellResult<()> {
        let color = match self.config.ui.prompt_color.as_str() {
            "green" => Color::Green,
            "blue" => Color::Blue,
            "red" => Color::Red,
            "yellow" => Color::Yellow,
            "cyan" => Color::Cyan,
            "magenta" => Color::Magenta,
            "white" => Color::White,
            _ => Color::Green,
        };

        io::stdout()
            .execute(SetForegroundColor(color))?
            .execute(Print("shell-t> "))?
            .execute(ResetColor)?;

        io::stdout().flush()?;
        Ok(())
    }

    /// Display plain text prompt
    fn display_plain_prompt(&self) -> ShellResult<()> {
        print!("shell-t> ");
        io::stdout().flush()?;
        Ok(())
    }

    /// Display a success message
    pub fn display_success(&self, message: &str) -> ShellResult<()> {
        if self.config.ui.enable_colors {
            io::stdout()
                .execute(SetForegroundColor(Color::Green))?
                .execute(Print(format!("✓ {}\n", message)))?
                .execute(ResetColor)?;
        } else {
            println!("✓ {}", message);
        }
        Ok(())
    }

    /// Display an error message
    pub fn display_error(&self, message: &str) -> ShellResult<()> {
        if self.config.ui.enable_colors {
            io::stdout()
                .execute(SetForegroundColor(Color::Red))?
                .execute(Print(format!("✗ {}\n", message)))?
                .execute(ResetColor)?;
        } else {
            eprintln!("✗ {}", message);
        }
        Ok(())
    }

    /// Display a warning message
    pub fn display_warning(&self, message: &str) -> ShellResult<()> {
        if self.config.ui.enable_colors {
            io::stdout()
                .execute(SetForegroundColor(Color::Yellow))?
                .execute(Print(format!("⚠ {}\n", message)))?
                .execute(ResetColor)?;
        } else {
            println!("⚠ {}", message);
        }
        Ok(())
    }

    /// Display informational message
    pub fn display_info(&self, message: &str) -> ShellResult<()> {
        if self.config.ui.enable_colors {
            io::stdout()
                .execute(SetForegroundColor(Color::Blue))?
                .execute(Print(format!("ℹ {}\n", message)))?
                .execute(ResetColor)?;
        } else {
            println!("ℹ {}", message);
        }
        Ok(())
    }

    /// Display a timestamped message if enabled
    pub fn display_timestamped(&self, message: &str) -> ShellResult<()> {
        if self.config.ui.show_timestamps {
            let now = chrono::Utc::now().format("%H:%M:%S");
            print!("[{}] ", now);
        }
        println!("{}", message);
        Ok(())
    }

    /// Clear the screen
    pub fn clear_screen(&self) -> ShellResult<()> {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()?;
        Ok(())
    }

    /// Move cursor to position
    pub fn move_cursor(&self, x: u16, y: u16) -> ShellResult<()> {
        use crossterm::cursor::MoveTo;
        io::stdout().execute(MoveTo(x, y))?;
        Ok(())
    }

    /// Get terminal size
    pub fn get_terminal_size(&self) -> ShellResult<(u16, u16)> {
        use crossterm::terminal::size;
        size().map_err(|e| crate::error::ShellError::Io(e))
    }
}

/// Progress indicator for long-running operations
pub struct ProgressIndicator {
    message: String,
    ui: UiManager,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(message: String, ui: UiManager) -> Self {
        Self { message, ui }
    }

    /// Start the progress indicator
    pub fn start(&self) -> ShellResult<()> {
        self.ui.display_info(&format!("Starting: {}", self.message))
    }

    /// Update progress
    pub fn update(&self, progress: f32) -> ShellResult<()> {
        let percentage = (progress * 100.0) as u32;
        print!("\r{}: {}%", self.message, percentage);
        io::stdout().flush()?;
        Ok(())
    }

    /// Complete the progress indicator
    pub fn complete(&self) -> ShellResult<()> {
        println!("\r{}: Complete ✓", self.message);
        Ok(())
    }

    /// Fail the progress indicator
    pub fn fail(&self, error: &str) -> ShellResult<()> {
        println!("\r{}: Failed ✗", self.message);
        self.ui.display_error(error)
    }
}

/// Table formatter for displaying structured data
pub struct TableFormatter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    ui: UiManager,
}

impl TableFormatter {
    /// Create a new table formatter
    pub fn new(headers: Vec<String>, ui: UiManager) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            ui,
        }
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    /// Display the table
    pub fn display(&self) -> ShellResult<()> {
        if self.headers.is_empty() && self.rows.is_empty() {
            return Ok(());
        }

        let mut col_widths = Vec::new();
        if !self.headers.is_empty() {
            for header in &self.headers {
                col_widths.push(header.len());
            }
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        if !self.headers.is_empty() {
            for (i, header) in self.headers.iter().enumerate() {
                if i > 0 { print!(" | "); }
                print!("{:<width$}", header, width = col_widths[i]);
            }
            println!();

            for (i, &width) in col_widths.iter().enumerate() {
                if i > 0 { print!("-+-"); }
                print!("{}", "-".repeat(width));
            }
            println!();
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 { print!(" | "); }
                print!("{:<width$}", cell, width = col_widths.get(i).copied().unwrap_or(10));
            }
            println!();
        }

        Ok(())
    }
}

/// Input reader with history and completion
pub struct InputReader {
    history: Vec<String>,
    history_index: usize,
    ui: UiManager,
}

impl InputReader {
    /// Create a new input reader
    pub fn new(ui: UiManager) -> Self {
        Self {
            history: Vec::new(),
            history_index: 0,
            ui,
        }
    }

    /// Read a line of input with basic editing
    pub fn read_line(&mut self, prompt: &str) -> ShellResult<String> {
        self.ui.display_info(prompt)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_string();

        if !input.is_empty() &&
           self.history.last().map_or(true, |last| last != &input) {
            self.history.push(input.clone());
            self.history_index = self.history.len();
        }

        Ok(input)
    }

    /// Get previous command from history
    pub fn previous_command(&mut self) -> Option<&String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.history.get(self.history_index)
        } else {
            None
        }
    }

    /// Get next command from history
    pub fn next_command(&mut self) -> Option<&String> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.history.get(self.history_index)
        } else {
            None
        }
    }

    /// Display command history
    pub fn display_history(&self) -> ShellResult<()> {
        for (i, cmd) in self.history.iter().enumerate() {
            println!("{:4} {}", i + 1, cmd);
        }
        Ok(())
    }
}