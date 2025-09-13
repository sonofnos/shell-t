use std::process::Stdio;

/// Represents a parsed command with its arguments and redirections
#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub input_redirect: Option<String>,
    pub output_redirect: Option<String>,
    pub append: bool,
    #[allow(dead_code)]
    pub background: bool,
}

/// Parse a command line string into a vector of Commands
pub fn parse_command(input: &str) -> Result<Vec<Command>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty command".to_string());
    }

    let pipe_commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();

    // Check for empty commands in pipeline
    for (i, cmd_str) in pipe_commands.iter().enumerate() {
        if cmd_str.is_empty() && i < pipe_commands.len() - 1 {
            return Err("Missing command after pipe".to_string());
        }
        if cmd_str.is_empty() && i == pipe_commands.len() - 1 && pipe_commands.len() > 1 {
            return Err("Missing command after pipe".to_string());
        }
    }

    let mut commands = Vec::new();

    for cmd_str in pipe_commands.iter() {
        let mut parts: Vec<String> = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        let chars = cmd_str.chars().peekable();

        for ch in chars {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                '"' | '\'' if in_quotes && ch == quote_char => {
                    in_quotes = false;
                    quote_char = ' ';
                }
                ' ' if !in_quotes => {
                    if !current_part.is_empty() {
                        parts.push(current_part.clone());
                        current_part.clear();
                    }
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        if parts.is_empty() {
            continue;
        }

        let mut program = String::new();
        let mut args = Vec::new();
        let mut input_redirect = None;
        let mut output_redirect = None;
        let mut append = false;
        let mut background = false;

        let mut i = 0;
        while i < parts.len() {
            let part = &parts[i];

            match part.as_str() {
                "<" => {
                    if i + 1 < parts.len() {
                        input_redirect = Some(parts[i + 1].clone());
                        i += 2;
                    } else {
                        return Err("Missing input file after '<'".to_string());
                    }
                }
                ">" => {
                    if i + 1 < parts.len() {
                        output_redirect = Some(parts[i + 1].clone());
                        append = false;
                        i += 2;
                    } else {
                        return Err("Missing output file after '>'".to_string());
                    }
                }
                ">>" => {
                    if i + 1 < parts.len() {
                        output_redirect = Some(parts[i + 1].clone());
                        append = true;
                        i += 2;
                    } else {
                        return Err("Missing output file after '>>'".to_string());
                    }
                }
                "&" => {
                    background = true;
                    i += 1;
                }
                _ => {
                    if program.is_empty() {
                        program = part.clone();
                    } else {
                        args.push(part.clone());
                    }
                    i += 1;
                }
            }
        }

        if program.is_empty() {
            return Err("No command specified".to_string());
        }

        commands.push(Command {
            program,
            args,
            input_redirect,
            output_redirect,
            append,
            background: background && i == pipe_commands.len() - 1,
        });
    }

    if commands.is_empty() {
        return Err("No commands to execute".to_string());
    }

    Ok(commands)
}

/// Get the standard input/output configuration for a command
#[allow(dead_code)]
pub fn get_stdio_config(cmd: &Command) -> (Stdio, Stdio, Stdio) {
    let stdin = if cmd.input_redirect.is_some() {
        Stdio::piped()
    } else {
        Stdio::inherit()
    };

    let stdout = if cmd.output_redirect.is_some() {
        Stdio::piped()
    } else {
        Stdio::inherit()
    };

    let stderr = Stdio::inherit();

    (stdin, stdout, stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let result = parse_command("ls -la");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "ls");
        assert_eq!(commands[0].args, vec!["-la"]);
        assert_eq!(commands[0].input_redirect, None);
        assert_eq!(commands[0].output_redirect, None);
        assert_eq!(commands[0].append, false);
    }

    #[test]
    fn test_parse_command_with_quotes() {
        let result = parse_command("echo \"hello world\"");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "echo");
        assert_eq!(commands[0].args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_command_with_single_quotes() {
        let result = parse_command("echo 'hello world'");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "echo");
        assert_eq!(commands[0].args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_input_redirection() {
        let result = parse_command("cat < input.txt");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "cat");
        assert_eq!(commands[0].args, Vec::<String>::new());
        assert_eq!(commands[0].input_redirect, Some("input.txt".to_string()));
    }

    #[test]
    fn test_parse_output_redirection() {
        let result = parse_command("echo hello > output.txt");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "echo");
        assert_eq!(commands[0].args, vec!["hello"]);
        assert_eq!(commands[0].output_redirect, Some("output.txt".to_string()));
        assert_eq!(commands[0].append, false);
    }

    #[test]
    fn test_parse_append_redirection() {
        let result = parse_command("echo hello >> output.txt");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "echo");
        assert_eq!(commands[0].args, vec!["hello"]);
        assert_eq!(commands[0].output_redirect, Some("output.txt".to_string()));
        assert_eq!(commands[0].append, true);
    }

    #[test]
    fn test_parse_pipeline() {
        let result = parse_command("ls -la | grep txt");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].program, "ls");
        assert_eq!(commands[0].args, vec!["-la"]);

        assert_eq!(commands[1].program, "grep");
        assert_eq!(commands[1].args, vec!["txt"]);
    }

    #[test]
    fn test_parse_complex_pipeline() {
        let result = parse_command("cat file.txt | grep error | sort | uniq > results.txt");
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 4);

        assert_eq!(commands[0].program, "cat");
        assert_eq!(commands[0].args, vec!["file.txt"]);

        assert_eq!(commands[1].program, "grep");
        assert_eq!(commands[1].args, vec!["error"]);

        assert_eq!(commands[2].program, "sort");
        assert_eq!(commands[2].args, Vec::<String>::new());

        assert_eq!(commands[3].program, "uniq");
        assert_eq!(commands[3].args, Vec::<String>::new());
        assert_eq!(commands[3].output_redirect, Some("results.txt".to_string()));
    }

    #[test]
    fn test_parse_empty_command() {
        let result = parse_command("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty command");
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse_command("   \t   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty command");
    }

    #[test]
    fn test_parse_redirection_without_file() {
        let result = parse_command("cat <");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing input file"));
    }

    #[test]
    fn test_parse_output_redirection_without_file() {
        let result = parse_command("echo >");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing output file"));
    }

    #[test]
    fn test_parse_append_redirection_without_file() {
        let result = parse_command("echo >>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing output file"));
    }

    #[test]
    fn test_parse_missing_command_after_pipe() {
        let result = parse_command("ls |");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing command after pipe");
    }

    #[test]
    fn test_get_stdio_config_no_redirection() {
        let cmd = Command {
            program: "ls".to_string(),
            args: vec![],
            input_redirect: None,
            output_redirect: None,
            append: false,
            background: false,
        };

        let (stdin, stdout, stderr) = get_stdio_config(&cmd);
        // Test that stdio config doesn't panic and returns valid values
        // We can't directly compare Stdio values, but we can verify the function works
        let _ = (stdin, stdout, stderr); // Just ensure values are returned
    }

    #[test]
    fn test_get_stdio_config_with_input_redirection() {
        let cmd = Command {
            program: "cat".to_string(),
            args: vec![],
            input_redirect: Some("input.txt".to_string()),
            output_redirect: None,
            append: false,
            background: false,
        };

        let (stdin, stdout, stderr) = get_stdio_config(&cmd);
        // Test that stdio config doesn't panic and returns valid values
        let _ = (stdin, stdout, stderr); // Just ensure values are returned
    }

    #[test]
    fn test_get_stdio_config_with_output_redirection() {
        let cmd = Command {
            program: "echo".to_string(),
            args: vec!["hello".to_string()],
            input_redirect: None,
            output_redirect: Some("output.txt".to_string()),
            append: false,
            background: false,
        };

        let (stdin, stdout, stderr) = get_stdio_config(&cmd);
        // Test that stdio config doesn't panic and returns valid values
        let _ = (stdin, stdout, stderr); // Just ensure values are returned
    }
}
