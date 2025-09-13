use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::{self, Write};
use std::process::Command;
use std::sync::Arc;

mod parser;
mod security;
mod builtins;
mod executor;
mod ui;
mod config;
mod error;

use error::ShellResult;

fn main() -> ShellResult<()> {
    println!("Shell-T - Secure Multi-Language Terminal");
    println!("Type 'exit' to quit\n");

    // Initialize configuration
    let config = config::Config::default();

    // Initialize security manager
    let security = Arc::new(security::SecurityManager::new());

    // Initialize managers
    let builtin_manager = builtins::BuiltinManager::new(Arc::clone(&security), config.clone());
    let executor = executor::CommandExecutor::new(Arc::clone(&security), config.clone());
    let ui_manager = ui::UiManager::new(config.clone());

    loop {
        // Display prompt using UI manager
        if let Err(e) = ui_manager.display_prompt() {
            eprintln!("UI error: {}", e);
            break;
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            println!("Goodbye!");
            break;
        }

        match parser::parse_command(input) {
            Ok(commands) => {
                if let Err(e) = execute_commands(&commands, &builtin_manager, &executor) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
        }
    }

    Ok(())
}

fn execute_commands(
    commands: &[parser::Command],
    builtin_manager: &builtins::BuiltinManager,
    executor: &executor::CommandExecutor,
) -> ShellResult<()> {
    if commands.is_empty() {
        return Ok(());
    }

    // Handle single command (no pipeline)
    if commands.len() == 1 {
        let cmd = &commands[0];
        if cmd.program.is_empty() {
            return Ok(());
        }

        // Try builtin commands first
        if let Some(result) = builtin_manager.execute_builtin(&cmd.program, &cmd.args)? {
            match result {
                builtins::BuiltinResult::Success(msg) => {
                    if let Some(msg) = msg {
                        println!("{}", msg);
                    }
                }
                builtins::BuiltinResult::Error(msg) => {
                    eprintln!("{}", msg);
                }
                builtins::BuiltinResult::Info(msg) => {
                    println!("{}", msg);
                }
                builtins::BuiltinResult::Warning(msg) => {
                    eprintln!("Warning: {}", msg);
                }
                builtins::BuiltinResult::Exit => {
                    std::process::exit(0);
                }
            }
            return Ok(());
        }

        // Not a builtin, execute as external command
        return executor.execute_pipeline(commands);
    }

    // Handle pipeline
    executor.execute_pipeline(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;

    fn create_test_managers() -> (builtins::BuiltinManager, executor::CommandExecutor) {
        let config = config::Config::default();
        let security = Arc::new(security::SecurityManager::new());
        let builtin_manager = builtins::BuiltinManager::new(Arc::clone(&security), config.clone());
        let executor = executor::CommandExecutor::new(security, config);
        (builtin_manager, executor)
    }

    #[test]
    fn test_execute_commands_empty() {
        let commands = Vec::new();
        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_commands_builtin_cd() {
        let original_dir = std::env::current_dir().unwrap();

        let commands = vec![parser::Command {
            program: "cd".to_string(),
            args: vec!["/tmp".to_string()],
            input_redirect: None,
            output_redirect: None,
            append: false,
            background: false,
        }];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());

        // Change back to original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_execute_commands_builtin_pwd() {
        let commands = vec![parser::Command {
            program: "pwd".to_string(),
            args: vec![],
            input_redirect: None,
            output_redirect: None,
            append: false,
            background: false,
        }];

        let result = execute_commands(&commands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_commands_external() {
        let commands = vec![parser::Command {
            program: "echo".to_string(),
            args: vec!["test".to_string()],
            input_redirect: None,
            output_redirect: None,
            append: false,
            background: false,
        }];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_commands_with_input_redirection() {
        // Create a temporary test file
        let test_content = "Hello, World!";
        fs::write("test_input.txt", test_content).unwrap();

        let commands = vec![parser::Command {
            program: "cat".to_string(),
            args: vec![],
            input_redirect: Some("test_input.txt".to_string()),
            output_redirect: None,
            append: false,
            background: false,
        }];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());

        // Clean up
        fs::remove_file("test_input.txt").unwrap();
    }

    #[test]
    fn test_execute_commands_with_output_redirection() {
        let commands = vec![parser::Command {
            program: "echo".to_string(),
            args: vec!["test output".to_string()],
            input_redirect: None,
            output_redirect: Some("test_output.txt".to_string()),
            append: false,
            background: false,
        }];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());

        // Verify file was created
        assert!(Path::new("test_output.txt").exists());

        // Clean up
        fs::remove_file("test_output.txt").unwrap();
    }

    #[test]
    fn test_execute_commands_with_append_redirection() {
        // Create initial file
        fs::write("test_append.txt", "initial content\n").unwrap();

        let commands = vec![parser::Command {
            program: "echo".to_string(),
            args: vec!["appended content".to_string()],
            input_redirect: None,
            output_redirect: Some("test_append.txt".to_string()),
            append: true,
            background: false,
        }];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());

        // Verify content was appended
        let content = fs::read_to_string("test_append.txt").unwrap();
        assert!(content.contains("initial content"));
        assert!(content.contains("appended content"));

        // Clean up
        fs::remove_file("test_append.txt").unwrap();
    }

    #[test]
    fn test_execute_commands_multiple_commands() {
        let commands = vec![
            parser::Command {
                program: "echo".to_string(),
                args: vec!["first".to_string()],
                input_redirect: None,
                output_redirect: None,
                append: false,
                background: false,
            },
            parser::Command {
                program: "echo".to_string(),
                args: vec!["second".to_string()],
                input_redirect: None,
                output_redirect: None,
                append: false,
                background: false,
            },
        ];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_commands_skip_empty_program() {
        let commands = vec![
            parser::Command {
                program: "".to_string(),
                args: vec![],
                input_redirect: None,
                output_redirect: None,
                append: false,
                background: false,
            },
            parser::Command {
                program: "echo".to_string(),
                args: vec!["test".to_string()],
                input_redirect: None,
                output_redirect: None,
                append: false,
                background: false,
            },
        ];

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_integration() {
        // Test that parser and executor work together
        let input = "echo hello world";
        let commands = parser::parse_command(input).unwrap();

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_integration_with_redirection() {
        let input = "echo test > output.txt";
        let commands = parser::parse_command(input).unwrap();

        let (builtin_manager, executor) = create_test_managers();
        let result = execute_commands(&commands, &builtin_manager, &executor);
        assert!(result.is_ok());

        // Verify file was created
        assert!(Path::new("output.txt").exists());

        // Clean up
        fs::remove_file("output.txt").unwrap();
    }
}
