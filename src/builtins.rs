use std::env;
use std::path::PathBuf;
use std::process;

use std::sync::Arc;
use crate::security::SecurityManager;
use crate::config::Config;
use crate::error::{ShellResult, ShellError};

/// Built-in command types
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinCommand {
    Cd,
    Pwd,
    Exit,
    Help,
    History,
    Alias,
    Unalias,
    Export,
    Unset,
    Jobs,
    Fg,
    Bg,
    Kill,
    Which,
    Type,
}

impl BuiltinCommand {
    /// Convert string to builtin command
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cd" => Some(BuiltinCommand::Cd),
            "pwd" => Some(BuiltinCommand::Pwd),
            "exit" => Some(BuiltinCommand::Exit),
            "help" => Some(BuiltinCommand::Help),
            "history" => Some(BuiltinCommand::History),
            "alias" => Some(BuiltinCommand::Alias),
            "unalias" => Some(BuiltinCommand::Unalias),
            "export" => Some(BuiltinCommand::Export),
            "unset" => Some(BuiltinCommand::Unset),
            "jobs" => Some(BuiltinCommand::Jobs),
            "fg" => Some(BuiltinCommand::Fg),
            "bg" => Some(BuiltinCommand::Bg),
            "kill" => Some(BuiltinCommand::Kill),
            "which" => Some(BuiltinCommand::Which),
            "type" => Some(BuiltinCommand::Type),
            _ => None,
        }
    }

    /// Check if a string represents a builtin command
    pub fn is_builtin(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
}

/// Result of executing a built-in command
#[derive(Debug)]
pub enum BuiltinResult {
    Success(Option<String>),
    Error(String),
    Info(String),
    Warning(String),
    Exit,
}

/// Manager for built-in commands
pub struct BuiltinManager {
    security: Arc<SecurityManager>,
    config: Config,
}

impl BuiltinManager {
    /// Create a new builtin manager
    pub fn new(security: Arc<SecurityManager>, config: Config) -> Self {
        Self { security, config }
    }

    /// Execute a built-in command
    pub fn execute_builtin(&self, command: &str, args: &[String]) -> ShellResult<Option<BuiltinResult>> {
        let builtin_cmd = match BuiltinCommand::from_str(command) {
            Some(cmd) => cmd,
            None => return Ok(None),
        };

        match builtin_cmd {
            BuiltinCommand::Cd => Ok(Some(self.execute_cd(args)?)),
            BuiltinCommand::Pwd => Ok(Some(self.execute_pwd()?)),
            BuiltinCommand::Exit => Ok(Some(BuiltinResult::Exit)),
            BuiltinCommand::Help => Ok(Some(self.execute_help()?)),
            BuiltinCommand::History => Ok(Some(self.execute_history()?)),
            BuiltinCommand::Alias => Ok(Some(self.execute_alias(args)?)),
            BuiltinCommand::Unalias => Ok(Some(self.execute_unalias(args)?)),
            BuiltinCommand::Export => Ok(Some(self.execute_export(args)?)),
            BuiltinCommand::Unset => Ok(Some(self.execute_unset(args)?)),
            BuiltinCommand::Jobs => Ok(Some(self.execute_jobs()?)),
            BuiltinCommand::Fg => Ok(Some(self.execute_fg(args)?)),
            BuiltinCommand::Bg => Ok(Some(self.execute_bg(args)?)),
            BuiltinCommand::Kill => Ok(Some(self.execute_kill(args)?)),
            BuiltinCommand::Which => Ok(Some(self.execute_which(args)?)),
            BuiltinCommand::Type => Ok(Some(self.execute_type(args)?)),
        }
    }

    /// Execute cd command
    fn execute_cd(&self, args: &[String]) -> ShellResult<BuiltinResult> {
        let path = if args.is_empty() {
            match env::var("HOME") {
                Ok(home) => home,
                Err(_) => return Ok(BuiltinResult::Error("HOME environment variable not set".to_string())),
            }
        } else {
            args[0].clone()
        };

        match env::set_current_dir(&path) {
            Ok(_) => Ok(BuiltinResult::Success(None)),
            Err(e) => Ok(BuiltinResult::Error(format!("cd: {}: {}", path, e))),
        }
    }

    /// Execute pwd command
    fn execute_pwd(&self) -> ShellResult<BuiltinResult> {
        match env::current_dir() {
            Ok(path) => Ok(BuiltinResult::Info(path.display().to_string())),
            Err(e) => Ok(BuiltinResult::Error(format!("pwd: {}", e))),
        }
    }

    /// Execute help command
    fn execute_help(&self) -> ShellResult<BuiltinResult> {
        let help_text = r#"Shell-T Built-in Commands:

Navigation:
  cd <dir>          Change directory
  pwd               Print working directory

Process Control:
  jobs              List background jobs
  fg [JOB]          Bring job to foreground
  bg [JOB]          Send job to background
  kill [PID]        Kill a process

Environment:
  export KEY=VALUE  Set environment variable
  unset KEY         Unset environment variable

Utilities:
  alias             Manage command aliases
  history           Show command history
  which COMMAND     Locate a command
  type COMMAND      Show command type
  help              Show this help
  exit              Exit the shell

Security Features:
- Input validation and sanitization
- Path traversal protection
- Resource limits and monitoring
- Command whitelisting/blacklisting
- Audit logging

For more information, see the documentation."#;

        Ok(BuiltinResult::Info(help_text.to_string()))
    }

    /// Execute history command
    fn execute_history(&self) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Command history not yet implemented".to_string()))
    }

    /// Execute alias command
    fn execute_alias(&self, _args: &[String]) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Command aliasing not yet implemented".to_string()))
    }

    /// Execute unalias command
    fn execute_unalias(&self, _args: &[String]) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Command unaliasing not yet implemented".to_string()))
    }

    /// Execute export command
    fn execute_export(&self, args: &[String]) -> ShellResult<BuiltinResult> {
        if args.is_empty() {
            return Ok(BuiltinResult::Error("export: missing argument".to_string()));
        }

        let arg = &args[0];
        if let Some(eq_pos) = arg.find('=') {
            let key = &arg[..eq_pos];
            let value = &arg[eq_pos + 1..];
            env::set_var(key, value);
            Ok(BuiltinResult::Success(None))
        } else {
            Ok(BuiltinResult::Error("export: invalid format, use KEY=VALUE".to_string()))
        }
    }

    /// Execute unset command
    fn execute_unset(&self, args: &[String]) -> ShellResult<BuiltinResult> {
        if args.is_empty() {
            return Ok(BuiltinResult::Error("unset: missing argument".to_string()));
        }

        env::remove_var(&args[0]);
        Ok(BuiltinResult::Success(None))
    }

    /// Execute jobs command
    fn execute_jobs(&self) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Background jobs not yet implemented".to_string()))
    }

    /// Execute fg command
    fn execute_fg(&self, _args: &[String]) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Foreground job control not yet implemented".to_string()))
    }

    /// Execute bg command
    fn execute_bg(&self, _args: &[String]) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Background job control not yet implemented".to_string()))
    }

    /// Execute kill command
    fn execute_kill(&self, _args: &[String]) -> ShellResult<BuiltinResult> {
        Ok(BuiltinResult::Info("Process killing not yet implemented".to_string()))
    }

    /// Execute which command
    fn execute_which(&self, args: &[String]) -> ShellResult<BuiltinResult> {
        if args.is_empty() {
            return Ok(BuiltinResult::Error("which: missing argument".to_string()));
        }

        match which::which(&args[0]) {
            Ok(path) => Ok(BuiltinResult::Info(path.display().to_string())),
            Err(_) => Ok(BuiltinResult::Error(format!("which: {}: command not found", args[0]))),
        }
    }

    /// Execute type command
    fn execute_type(&self, args: &[String]) -> ShellResult<BuiltinResult> {
        if args.is_empty() {
            return Ok(BuiltinResult::Error("type: missing argument".to_string()));
        }

        let cmd = &args[0];
        if BuiltinCommand::is_builtin(cmd) {
            Ok(BuiltinResult::Info(format!("{} is a shell builtin", cmd)))
        } else {
            match which::which(cmd) {
                Ok(path) => Ok(BuiltinResult::Info(format!("{} is {}", cmd, path.display()))),
                Err(_) => Ok(BuiltinResult::Error(format!("type: {}: not found", cmd))),
            }
        }
    }
}

