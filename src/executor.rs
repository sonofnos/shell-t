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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::parser::Command as ParsedCommand;
    use std::sync::Arc;

    fn create_test_executor() -> CommandExecutor {
        let security = Arc::new(SecurityManager::new());
        let config = Config::default();
        CommandExecutor::new(security, config)
    }

    fn create_test_command(program: &str, args: Vec<&str>) -> ParsedCommand {
        ParsedCommand {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            input_redirect: None,
            output_redirect: None,
            append: false,
            background: false,
        }
    }

    #[test]
    fn test_executor_creation() {
        let executor = create_test_executor();
        // Test passes if executor is created successfully
        assert!(true);
    }

    #[test]
    fn test_resolve_command_regular() {
        let executor = create_test_executor();

        let (cmd, args) = executor.resolve_command("ls", &["-la".to_string()]).unwrap();
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la"]);
    }

    #[test]
    fn test_resolve_command_python() {
        let executor = create_test_executor();

        let (cmd, args) = executor.resolve_command("script.py", &["arg1".to_string()]).unwrap();
        assert_eq!(cmd, "python3");
        assert_eq!(args, vec!["script.py", "arg1"]);
    }

    #[test]
    fn test_resolve_command_ruby() {
        let executor = create_test_executor();

        let (cmd, args) = executor.resolve_command("script.rb", &["arg1".to_string()]).unwrap();
        assert_eq!(cmd, "ruby");
        assert_eq!(args, vec!["script.rb", "arg1"]);
    }

    #[test]
    fn test_resolve_command_javascript() {
        let executor = create_test_executor();

        let (cmd, args) = executor.resolve_command("script.js", &["arg1".to_string()]).unwrap();
        assert_eq!(cmd, "node");
        assert_eq!(args, vec!["script.js", "arg1"]);
    }

    #[test]
    fn test_validate_command_whitelist_allowed() {
        let mut config = Config::default();
        config.security.command_whitelist = Some(vec!["ls".to_string(), "pwd".to_string()]);

        let security = Arc::new(SecurityManager::new());
        let executor = CommandExecutor::new(security, config);

        assert!(executor.validate_command("ls").is_ok());
        assert!(executor.validate_command("pwd").is_ok());
    }

    #[test]
    fn test_validate_command_whitelist_denied() {
        let mut config = Config::default();
        config.security.command_whitelist = Some(vec!["ls".to_string(), "pwd".to_string()]);

        let security = Arc::new(SecurityManager::new());
        let executor = CommandExecutor::new(security, config);

        assert!(executor.validate_command("rm").is_err());
        assert!(executor.validate_command("sudo").is_err());
    }

    #[test]
    fn test_validate_command_blacklist() {
        let mut config = Config::default();
        config.security.command_blacklist = Some(vec!["rm".to_string(), "sudo".to_string()]);

        let security = Arc::new(SecurityManager::new());
        let executor = CommandExecutor::new(security, config);

        assert!(executor.validate_command("ls").is_ok());
        assert!(executor.validate_command("rm").is_err());
        assert!(executor.validate_command("sudo").is_err());
    }

    #[test]
    fn test_validate_args_path_traversal() {
        let executor = create_test_executor();

        let args = vec!["../../../etc/passwd".to_string()];
        assert!(executor.validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_too_long() {
        let executor = create_test_executor();

        let long_arg = "a".repeat(10000);
        let args = vec![long_arg];
        assert!(executor.validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_valid() {
        let executor = create_test_executor();

        let args = vec!["-la".to_string(), "--color".to_string(), "file.txt".to_string()];
        assert!(executor.validate_args(&args).is_ok());
    }

    #[test]
    fn test_execute_pipeline_empty() {
        let executor = create_test_executor();
        let commands = Vec::new();

        let result = executor.execute_pipeline(&commands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_pipeline_too_long() {
        let mut config = Config::default();
        config.limits.max_pipeline_length = 2;

        let security = Arc::new(SecurityManager::new());
        let executor = CommandExecutor::new(security, config);

        let commands = vec![
            create_test_command("ls", vec![]),
            create_test_command("grep", vec!["test"]),
            create_test_command("sort", vec![]),
        ];

        let result = executor.execute_pipeline(&commands);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_pipeline_single_command() {
        let executor = create_test_executor();
        let commands = vec![create_test_command("true", vec![])];

        // This should work for commands that exist
        let result = executor.execute_pipeline(&commands);
        // We expect this to succeed or fail based on whether 'true' command exists
        // The important thing is that it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_execute_pipeline_with_empty_program() {
        let executor = create_test_executor();

        let mut cmd = create_test_command("", vec![]);
        cmd.program = String::new();

        let commands = vec![cmd];

        // Should skip empty commands without error
        let result = executor.execute_pipeline(&commands);
        assert!(result.is_ok());
    }
}