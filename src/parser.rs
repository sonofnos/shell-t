use std::process::Stdio;

/// Represents a parsed command with its arguments and redirections
#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub input_redirect: Option<String>,
    pub output_redirect: Option<String>,
    pub append: bool,
    pub background: bool,
}

/// Parse a command line string into a vector of Commands
pub fn parse_command(input: &str) -> Result<Vec<Command>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty command".to_string());
    }

    let pipe_commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();

    let mut commands = Vec::new();

    for (_i, cmd_str) in pipe_commands.iter().enumerate() {
        let mut parts: Vec<String> = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        let mut chars = cmd_str.chars().peekable();

        while let Some(ch) = chars.next() {
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