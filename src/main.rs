use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::{self, Write};
use std::process::Command;

mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Shell-T - Secure Multi-Language Terminal");
    println!("Type 'exit' to quit\n");

    loop {
        io::stdout()
            .execute(SetForegroundColor(Color::Green))?
            .execute(Print("shell-t> "))?
            .execute(ResetColor)?;
        io::stdout().flush()?;

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
                if let Err(e) = execute_commands(&commands) {
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

fn execute_commands(commands: &[parser::Command]) -> Result<(), Box<dyn std::error::Error>> {
    if commands.is_empty() {
        return Ok(());
    }

    for cmd in commands {
        if cmd.program.is_empty() {
            continue;
        }

        match cmd.program.as_str() {
            "cd" => {
                if let Some(dir) = cmd.args.first() {
                    std::env::set_current_dir(dir)?;
                }
                continue;
            }
            "pwd" => {
                println!("{}", std::env::current_dir()?.display());
                continue;
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => {}
        }

        let mut command = Command::new(&cmd.program);
        command.args(&cmd.args);

        if let Some(input_file) = &cmd.input_redirect {
            let file = std::fs::File::open(input_file)?;
            command.stdin(file);
        }

        if let Some(output_file) = &cmd.output_redirect {
            let file = if cmd.append {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output_file)?
            } else {
                std::fs::File::create(output_file)?
            };
            command.stdout(file);
        }

        let status = command.status()?;
        if !status.success() {
            eprintln!("Command failed with exit code: {}", status);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_execute_commands_empty() {
        let commands = Vec::new();
        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
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

        let result = execute_commands(&commands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_integration() {
        // Test that parser and executor work together
        let input = "echo hello world";
        let commands = parser::parse_command(input).unwrap();

        let result = execute_commands(&commands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_integration_with_redirection() {
        let input = "echo test > output.txt";
        let commands = parser::parse_command(input).unwrap();

        let result = execute_commands(&commands);
        assert!(result.is_ok());

        // Verify file was created
        assert!(Path::new("output.txt").exists());

        // Clean up
        fs::remove_file("output.txt").unwrap();
    }
}
