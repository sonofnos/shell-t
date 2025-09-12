use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::{SecurityError, ShellResult};
use crate::config::Config;

/// Global security state
pub struct SecurityManager {
    active_processes: AtomicUsize,
    command_history: Mutex<HashMap<String, CommandStats>>,
    rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
}

#[derive(Debug, Clone)]
struct CommandStats {
    count: usize,
    last_execution: Instant,
    total_time: Duration,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            active_processes: AtomicUsize::new(0),
            command_history: Mutex::new(HashMap::new()),
            rate_limiter: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a new process can be started
    pub fn can_start_process(&self, config: &Config) -> ShellResult<()> {
        let current = self.active_processes.load(Ordering::SeqCst);
        if current >= config.limits.max_background_processes {
            return Err(SecurityError::ResourceLimitExceeded(
                "Maximum background processes reached".to_string()
            ).into());
        }
        Ok(())
    }

    /// Register a new process
    pub fn register_process(&self) -> ProcessGuard {
        self.active_processes.fetch_add(1, Ordering::SeqCst);
        ProcessGuard {
            manager: self,
        }
    }

    /// Check rate limiting for a user/command combination
    pub fn check_rate_limit(&self, key: &str, config: &Config) -> ShellResult<()> {
        let mut limiter = self.rate_limiter.lock().unwrap();
        let now = Instant::now();

        let entries = limiter.entry(key.to_string()).or_insert_with(Vec::new);

        entries.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

        if entries.len() >= 10 {
            return Err(SecurityError::ResourceLimitExceeded(
                "Rate limit exceeded".to_string()
            ).into());
        }

        entries.push(now);
        Ok(())
    }

    /// Record command execution for monitoring
    pub fn record_command(&self, command: &str, execution_time: Duration) {
        let mut history = self.command_history.lock().unwrap();
        let stats = history.entry(command.to_string()).or_insert(CommandStats {
            count: 0,
            last_execution: Instant::now(),
            total_time: Duration::new(0, 0),
        });

        stats.count += 1;
        stats.last_execution = Instant::now();
        stats.total_time += execution_time;
    }

    /// Validate user input for security violations
    pub fn validate_input(&self, input: &str) -> ShellResult<()> {
        if input.contains('\0') {
            return Err(SecurityError::InvalidInput("Null bytes not allowed".to_string()).into());
        }

        if input.len() > 10000 {
            return Err(SecurityError::InvalidInput("Input too long".to_string()).into());
        }

        let suspicious_patterns = [
            ";", "&", "|", "`", "$", "(", ")", "<", ">", "\"", "'", "\\",
            "rm ", "del ", "format ", "shutdown", "reboot", "halt",
            "../", "..\\", "/etc/", "/bin/", "/usr/", "C:\\",
        ];

        for pattern in &suspicious_patterns {
            if input.contains(pattern) {
                return Err(SecurityError::InvalidInput(
                    format!("Suspicious pattern detected: {}", pattern)
                ).into());
            }
        }

        Ok(())
    }
}

/// RAII guard for process management
pub struct ProcessGuard<'a> {
    manager: &'a SecurityManager,
}

impl<'a> Drop for ProcessGuard<'a> {
    fn drop(&mut self) {
        self.manager.active_processes.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Input validation and sanitization
pub mod validation {
    use super::*;
    use regex::Regex;
    use std::ffi::OsStr;

    /// Validate and sanitize user input
    pub fn sanitize_input(input: &str, config: &Config) -> ShellResult<String> {
        if !config.security.sanitize_input {
            return Ok(input.to_string());
        }

        let mut sanitized = input.to_string();

        sanitized = sanitized.replace('\0', "");

        sanitized = sanitized.chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
            .collect();

        check_suspicious_patterns(&sanitized)?;

        if sanitized.len() > config.security.max_command_length {
            sanitized = sanitized[..config.security.max_command_length].to_string();
        }

        Ok(sanitized)
    }

    /// Check for suspicious patterns that might indicate attacks
    fn check_suspicious_patterns(input: &str) -> ShellResult<()> {
        let suspicious_patterns = [
            r"\$\(.*\)",
            r"`.*`",
            r"\$\{.*\}",
            r";.*;",
            r"&&.*&&",
            r"\|\|.*\|\|",
        ];

        for pattern in &suspicious_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(input) {
                    return Err(SecurityError::DangerousCommand(
                        format!("Suspicious pattern detected: {}", pattern)
                    ).into());
                }
            }
        }

        Ok(())
    }

    /// Validate file path for security
    pub fn validate_file_path(path: &str, config: &Config) -> ShellResult<PathBuf> {
        if !config.security.validate_paths {
            return Ok(PathBuf::from(path));
        }

        let path_buf = PathBuf::from(path);

        if path.contains("..") {
            return Err(SecurityError::PathTraversal(path.to_string()).into());
        }

        if path_buf.is_absolute() {
            let allowed_dirs = ["/tmp", "/var/tmp", "/home", "/Users"];
            let is_allowed = allowed_dirs.iter().any(|dir| path.starts_with(dir));

            if !is_allowed {
                return Err(SecurityError::PathTraversal(
                    "Absolute path not in allowed directories".to_string()
                ).into());
            }
        }

        if let Ok(metadata) = path_buf.metadata() {
            if metadata.permissions().readonly() {
            } else {
            }
        }

        Ok(path_buf)
    }

    /// Validate command arguments
    pub fn validate_arguments(args: &[String], config: &Config) -> ShellResult<()> {
        if args.len() > config.security.max_arg_count {
            return Err(SecurityError::InvalidInput("Too many arguments".to_string()).into());
        }

        for arg in args {
            if arg.len() > config.security.max_command_length {
                return Err(SecurityError::InvalidInput("Argument too long".to_string()).into());
            }

            let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '<', '>', '\\'];
            if arg.chars().any(|c| dangerous_chars.contains(&c)) {
                return Err(SecurityError::DangerousCommand(
                    format!("Dangerous character in argument: {}", arg)
                ).into());
            }
        }

        Ok(())
    }
}

/// Process monitoring and resource management
pub mod monitoring {
    use super::*;
    use std::sync::Arc;
    use tokio::time::timeout;

    /// Execute a command with resource monitoring
    pub async fn execute_with_monitoring(
        command: &str,
        args: &[String],
        config: &Config,
        security_manager: Arc<SecurityManager>,
    ) -> ShellResult<process::Output> {
        security_manager.check_rate_limit(&format!("cmd:{}", command), config)?;

        security_manager.can_start_process(config)?;

        let start_time = Instant::now();

        let result = timeout(
            Duration::from_secs(config.limits.command_timeout),
            tokio::process::Command::new(command)
                .args(args)
                .output()
        ).await;

        let execution_time = start_time.elapsed();

        security_manager.record_command(command, execution_time);

        match result {
            Ok(output_result) => {
                match output_result {
                    Ok(output) => {
                        if output.stdout.len() > config.limits.max_memory_mb * 1024 * 1024 {
                            return Err(SecurityError::ResourceLimitExceeded(
                                "Output too large".to_string()
                            ).into());
                        }
                        Ok(output)
                    }
                    Err(e) => Err(crate::error::ShellError::CommandExecution(e.to_string())),
                }
            }
            Err(_) => Err(SecurityError::ResourceLimitExceeded(
                "Command execution timeout".to_string()
            ).into()),
        }
    }
}

/// Environment security
pub mod environment {
    use super::*;
    use std::env;

    /// Sanitize environment variables
    pub fn sanitize_environment() -> ShellResult<()> {
        let dangerous_vars = [
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "PATH",
            "SHELL",
            "BASH_ENV",
            "ENV",
        ];

        for var in &dangerous_vars {
            env::remove_var(var);
        }

        env::set_var("PATH", "/usr/local/bin:/usr/bin:/bin");
        env::set_var("SHELL", "/bin/sh");

        Ok(())
    }

    /// Validate environment before command execution
    pub fn validate_environment() -> ShellResult<()> {
        if is_elevated_privileges() {
            return Err(SecurityError::PermissionDenied(
                "Running with elevated privileges".to_string()
            ).into());
        }

        for (key, value) in env::vars() {
            if key.contains("LD_") || key.contains("DYLD_") {
                return Err(SecurityError::DangerousCommand(
                    format!("Suspicious environment variable: {}", key)
                ).into());
            }

            if value.contains('\0') {
                return Err(SecurityError::InvalidInput(
                    format!("Null byte in environment variable: {}", key)
                ).into());
            }
        }

        Ok(())
    }

    /// Check if running with elevated privileges
    fn is_elevated_privileges() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }

        #[cfg(not(unix))]
        {
            false
        }
    }
}

/// Environment security
pub mod environment {
    use super::*;
    use std::env;

    /// Sanitize environment variables
    pub fn sanitize_environment() -> ShellResult<()> {
        // Remove potentially dangerous environment variables
        let dangerous_vars = [
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "PATH",  // We'll set a safe PATH instead
            "SHELL",
            "BASH_ENV",
            "ENV",
        ];

        for var in &dangerous_vars {
            env::remove_var(var);
        }

        // Set safe PATH
        env::set_var("PATH", "/usr/local/bin:/usr/bin:/bin");

        // Set safe shell
        env::set_var("SHELL", "/bin/sh");

        Ok(())
    }

    /// Validate environment before command execution
    pub fn validate_environment() -> ShellResult<()> {
        // Check if we're running with elevated privileges
        if is_elevated_privileges() {
            return Err(SecurityError::PermissionDenied(
                "Running with elevated privileges".to_string()
            ).into());
        }

        // Check for suspicious environment variables
        for (key, value) in env::vars() {
            if key.contains("LD_") || key.contains("DYLD_") {
                return Err(SecurityError::DangerousCommand(
                    format!("Suspicious environment variable: {}", key)
                ).into());
            }

            if value.contains('\0') {
                return Err(SecurityError::InvalidInput(
                    format!("Null byte in environment variable: {}", key)
                ).into());
            }
        }

        Ok(())
    }

    /// Check if running with elevated privileges
    fn is_elevated_privileges() -> bool {
        // On Unix-like systems, check if effective UID is 0
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }

        #[cfg(not(unix))]
        {
            false
        }
    }
}