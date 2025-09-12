use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

use crate::config::Config;
use crate::error::{ShellError, ShellResult};
use crate::parser::Command as ParsedCommand;
use crate::security::SecurityManager;

/// Command execution engine
pub struct CommandExecutor {
    security: Arc<SecurityManager>,
    config: Config,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(security: Arc<SecurityManager>, config: Config) -> Self {
        Self { security, config }
    }

    /// Execute a pipeline of commands
    pub fn execute_pipeline(&self, commands: &[ParsedCommand]) -> ShellResult<()> {
        if commands.is_empty() {
            return Ok(());
        }

        if commands.len() > self.config.limits.max_pipeline_length {
            return Err(ShellError::Process("Pipeline too long".to_string()));
        }

        let mut children = Vec::new();
        let mut prev_stdout = None;

        for (i, cmd) in commands.iter().enumerate() {
            if cmd.program.is_empty() {
                continue;
            }

            let (actual_cmd, actual_args) = self.resolve_command(&cmd.program, &cmd.args)?;

            self.validate_command(&actual_cmd)?;
            self.validate_args(&actual_args)?;

            let mut command = Command::new(&actual_cmd);
            command.args(&actual_args);

            if let Some(prev) = prev_stdout.take() {
                command.stdin(prev);
            } else if let Some(ref input_file) = cmd.input_redirect {
                match std::fs::File::open(input_file) {
                    Ok(file) => { command.stdin(file); }
                    Err(e) => {
                        return Err(ShellError::FileSystem(format!("Error opening input file {}: {}", input_file, e)));
                    }
                }
            }

            if i < commands.len() - 1 {
                command.stdout(Stdio::piped());
            } else if let Some(ref output_file) = cmd.output_redirect {
                match if cmd.append {
                    std::fs::OpenOptions::new().create(true).append(true).open(output_file)
                } else {
                    std::fs::File::create(output_file)
                } {
                    Ok(file) => { command.stdout(file); }
                    Err(e) => {
                        return Err(ShellError::FileSystem(format!("Error opening output file {}: {}", output_file, e)));
                    }
                }
            }

            let start_time = Instant::now();

            match command.spawn() {
                Ok(mut child) => {
                    if i < commands.len() - 1 {
                        prev_stdout = child.stdout.take();
                    }
                    children.push(child);

                    let execution_time = start_time.elapsed();
                    self.security.record_command(&actual_cmd, execution_time);
                }
                Err(e) => {
                    return Err(ShellError::CommandExecution(format!("Failed to execute {}: {}", actual_cmd, e)));
                }
            }
        }

        if !commands.last().map_or(false, |c| c.background) {
            for mut child in children {
                if let Err(e) = child.wait() {
                    return Err(ShellError::Process(format!("Process wait error: {}", e)));
                }
            }
        }

        Ok(())
    }

    /// Resolve command name to actual executable
    fn resolve_command(&self, program: &str, args: &[String]) -> ShellResult<(String, Vec<String>)> {
        if program.ends_with(".py") {
            Ok(("python3".to_string(), vec![program.to_string()].into_iter().chain(args.iter().cloned()).collect()))
        } else if program.ends_with(".rb") {
            Ok(("ruby".to_string(), vec![program.to_string()].into_iter().chain(args.iter().cloned()).collect()))
        } else if program.ends_with(".js") {
            Ok(("node".to_string(), vec![program.to_string()].into_iter().chain(args.iter().cloned()).collect()))
        } else {
            Ok((program.to_string(), args.to_vec()))
        }
    }

    /// Validate a command against security policies
    fn validate_command(&self, command: &str) -> ShellResult<()> {
        if let Some(ref whitelist) = self.config.security.command_whitelist {
            if !whitelist.contains(&command.to_string()) {
                return Err(ShellError::SecurityViolation(format!("Command not in whitelist: {}", command)));
            }
        }

        if let Some(ref blacklist) = self.config.security.command_blacklist {
            if blacklist.contains(&command.to_string()) {
                return Err(ShellError::SecurityViolation(format!("Command blacklisted: {}", command)));
            }
        }

        Ok(())
    }

    /// Validate command arguments
    fn validate_args(&self, args: &[String]) -> ShellResult<()> {
        for arg in args {
            if arg.contains("../") || arg.contains("..\\") {
                return Err(ShellError::SecurityViolation("Path traversal detected".to_string()));
            }

            if arg.len() > self.config.limits.max_arg_length {
                return Err(ShellError::SecurityViolation("Argument too long".to_string()));
            }
        }
        Ok(())
    }
}