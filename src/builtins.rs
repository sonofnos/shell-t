use std::env;
use std::path::PathBuf;
use std::process;

use std::sync::Arc;
use crate::security::SecurityManager;

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

use std::env;
use std::path::PathBuf;
use std::process;

use std::sync::Arc;
use crate::security::SecurityManager;

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

/// Result of executing a built-in command
#[derive(Debug)]
pub enum BuiltinResult {
    /// Command executed successfully with optional message
    Success(Option<String>),
    /// Command failed with error message
    Error(String),
    /// Command produced informational output
    Info(String),
    /// Command produced warning output
    Warning(String),
    /// Command indicates shell should exit
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
            // Go to home directory
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
  cd [DIR]          Change directory
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

impl BuiltinCommand {
    /// Parse a command string to determine if it's a built-in
    pub fn from_str(cmd: &str) -> Option<Self> {
        match cmd {
            "cd" => Some(Self::Cd),
            "pwd" => Some(Self::Pwd),
            "exit" => Some(Self::Exit),
            "help" => Some(Self::Help),
            "history" => Some(Self::History),
            "alias" => Some(Self::Alias),
            "unalias" => Some(Self::Unalias),
            "export" => Some(Self::Export),
            "unset" => Some(Self::Unset),
            "jobs" => Some(Self::Jobs),
            "fg" => Some(Self::Fg),
            "bg" => Some(Self::Bg),
            "kill" => Some(Self::Kill),
            "which" => Some(Self::Which),
            "type" => Some(Self::Type),
            _ => None,
        }
    }

    /// Check if a command is a built-in
    pub fn is_builtin(cmd: &str) -> bool {
        Self::from_str(cmd).is_some()
    }
}

/// Execute a built-in command
pub fn execute_builtin(
    command: BuiltinCommand,
    args: &[String],
    config: &Config,
) -> ShellResult<Option<i32>> {
    match command {
        BuiltinCommand::Cd => execute_cd(args, config),
        BuiltinCommand::Pwd => execute_pwd(),
        BuiltinCommand::Exit => execute_exit(args),
        BuiltinCommand::Help => execute_help(),
        BuiltinCommand::History => execute_history(),
        BuiltinCommand::Alias => execute_alias(args),
        BuiltinCommand::Unalias => execute_unalias(args),
        BuiltinCommand::Export => execute_export(args),
        BuiltinCommand::Unset => execute_unset(args),
        BuiltinCommand::Jobs => execute_jobs(),
        BuiltinCommand::Fg => execute_fg(args),
        BuiltinCommand::Bg => execute_bg(args),
        BuiltinCommand::Kill => execute_kill(args),
        BuiltinCommand::Which => execute_which(args),
        BuiltinCommand::Type => execute_type(args),
    }
}

/// Change directory command
fn execute_cd(args: &[String], config: &Config) -> ShellResult<Option<i32>> {
    let path = match args.get(0) {
        Some(p) => p,
        None => {
            match env::var("HOME") {
                Ok(home) => home,
                Err(_) => return Err(ShellError::FileSystem("HOME not set".to_string())),
            }
        }
    };

    validation::validate_file_path(path, config)?;

    match env::set_current_dir(path) {
        Ok(_) => {
            println!("Changed directory to: {}", path);
            Ok(Some(0))
        }
        Err(e) => Err(ShellError::FileSystem(format!("cd: {}: {}", path, e))),
    }
}

/// Print working directory command
fn execute_pwd() -> ShellResult<Option<i32>> {
    match env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
            Ok(Some(0))
        }
        Err(e) => Err(ShellError::FileSystem(format!("pwd: {}", e))),
    }
}

/// Exit command
fn execute_exit(args: &[String]) -> ShellResult<Option<i32>> {
    let code = args.get(0)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    println!("Exiting with code: {}", code);
    process::exit(code);
}

/// Help command
fn execute_help() -> ShellResult<Option<i32>> {
    println!("Shell-T - Secure Terminal Shell");
    println!("===============================");
    println!();
    println!("Built-in commands:");
    println!("  cd <dir>     - Change directory");
    println!("  pwd          - Print working directory");
    println!("  exit [code]  - Exit the shell");
    println!("  help         - Show this help");
    println!("  history      - Show command history");
    println!("  alias        - Manage aliases");
    println!("  export       - Set environment variables");
    println!("  jobs         - List background jobs");
    println!("  fg <job>     - Bring job to foreground");
    println!("  bg <job>     - Send job to background");
    println!("  kill <job>   - Kill a job");
    println!("  which <cmd>  - Locate a command");
    println!("  type <cmd>   - Show command type");
    println!();
    println!("Security features:");
    println!("  Input validation and sanitization");
    println!("  Path traversal protection");
    println!("  Command whitelisting/blacklisting");
    println!("  Resource limits and monitoring");
    println!("  Audit logging");

    Ok(Some(0))
}

/// History command (placeholder)
fn execute_history() -> ShellResult<Option<i32>> {
    println!("Command history not yet implemented");
    Ok(Some(0))
}

/// Alias command (placeholder)
fn execute_alias(args: &[String]) -> ShellResult<Option<i32>> {
    if args.is_empty() {
        println!("No aliases defined");
    } else {
        println!("Alias management not yet implemented");
    }
    Ok(Some(0))
}

/// Unalias command (placeholder)
fn execute_unalias(args: &[String]) -> ShellResult<Option<i32>> {
    println!("Unalias not yet implemented");
    Ok(Some(0))
}

/// Export command
fn execute_export(args: &[String]) -> ShellResult<Option<i32>> {
    if args.is_empty() {
        for (key, value) in env::vars() {
            println!("{}={}", key, value);
        }
    } else {
        for arg in args {
            if let Some(eq_pos) = arg.find('=') {
                let key = &arg[..eq_pos];
                let value = &arg[eq_pos + 1..];

                if key.is_empty() {
                    return Err(ShellError::Config("Empty variable name".to_string()));
                }

                if key.chars().any(|c| !c.is_alphanumeric() && c != '_') {
                    return Err(ShellError::Config(format!("Invalid variable name: {}", key)));
                }

                env::set_var(key, value);
                println!("Exported: {}={}", key, value);
            } else {
                return Err(ShellError::Config(format!("Invalid export syntax: {}", arg)));
            }
        }
    }
    Ok(Some(0))
}

/// Unset command
fn execute_unset(args: &[String]) -> ShellResult<Option<i32>> {
    for arg in args {
        env::remove_var(arg);
        println!("Unset: {}", arg);
    }
    Ok(Some(0))
}

/// Jobs command (placeholder)
fn execute_jobs() -> ShellResult<Option<i32>> {
    println!("No background jobs");
    Ok(Some(0))
}

/// Foreground command (placeholder)
fn execute_fg(args: &[String]) -> ShellResult<Option<i32>> {
    println!("Foreground job control not yet implemented");
    Ok(Some(0))
}

/// Background command (placeholder)
fn execute_bg(args: &[String]) -> ShellResult<Option<i32>> {
    println!("Background job control not yet implemented");
    Ok(Some(0))
}

/// Kill command (placeholder)
fn execute_kill(args: &[String]) -> ShellResult<Option<i32>> {
    println!("Kill command not yet implemented");
    Ok(Some(0))
}

/// Which command
fn execute_which(args: &[String]) -> ShellResult<Option<i32>> {
    if args.is_empty() {
        return Err(ShellError::Config("which: missing argument".to_string()));
    }

    for arg in args {
        if let Ok(path) = which::which(arg) {
            println!("{}", path.display());
        } else {
            println!("which: {}: command not found", arg);
        }
    }
    Ok(Some(0))
}

/// Type command
fn execute_type(args: &[String]) -> ShellResult<Option<i32>> {
    if args.is_empty() {
        return Err(ShellError::Config("type: missing argument".to_string()));
    }

    for arg in args {
        if BuiltinCommand::is_builtin(arg) {
            println!("{} is a shell builtin", arg);
        } else if which::which(arg).is_ok() {
            println!("{} is {}", arg, which::which(arg).unwrap().display());
        } else {
            println!("{}: not found", arg);
        }
    }
    Ok(Some(0))
}