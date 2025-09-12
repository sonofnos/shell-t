use std::fmt;
use std::io;

/// Custom error type for Shell-T operations
#[derive(Debug)]
pub enum ShellError {
    Io(io::Error),
    CommandExecution(String),
    Parse(String),
    SecurityViolation(String),
    Config(String),
    FileSystem(String),
    Process(String),
}

/// Security-specific error types
#[derive(Debug)]
pub enum SecurityError {
    PathTraversal(String),
    DangerousCommand(String),
    InvalidInput(String),
    PermissionDenied(String),
    ResourceLimitExceeded(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::Io(err) => write!(f, "I/O error: {}", err),
            ShellError::CommandExecution(msg) => write!(f, "Command execution failed: {}", msg),
            ShellError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ShellError::Security(err) => write!(f, "Security error: {}", err),
            ShellError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ShellError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            ShellError::Process(msg) => write!(f, "Process error: {}", msg),
        }
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::PathTraversal(path) => {
                write!(f, "Path traversal attempt detected: {}", path)
            }
            SecurityError::DangerousCommand(cmd) => {
                write!(f, "Dangerous command blocked: {}", cmd)
            }
            SecurityError::InvalidInput(input) => {
                write!(f, "Invalid input: {}", input)
            }
            SecurityError::PermissionDenied(resource) => {
                write!(f, "Permission denied: {}", resource)
            }
            SecurityError::ResourceLimitExceeded(limit) => {
                write!(f, "Resource limit exceeded: {}", limit)
            }
        }
    }
}

impl std::error::Error for ShellError {}
impl std::error::Error for SecurityError {}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::Io(err)
    }
}

impl From<SecurityError> for ShellError {
    fn from(err: SecurityError) -> Self {
        ShellError::Security(err)
    }
}

/// Result type alias for Shell operations
pub type ShellResult<T> = Result<T, ShellError>;

/// Security validation functions
pub mod security {
    use super::{SecurityError, ShellResult};
    use std::path::Path;

    /// Validate that a path doesn't contain path traversal attempts
    pub fn validate_path(path: &str) -> ShellResult<()> {
        let path_obj = Path::new(path);

        if path.contains("..") || path.contains("../") || path.starts_with('/') {
            if path.starts_with('/') && !is_allowed_absolute_path(path) {
                return Err(SecurityError::PathTraversal(path.to_string()).into());
            }
        }

        if path.contains('\0') {
            return Err(SecurityError::InvalidInput("Null byte detected".to_string()).into());
        }

        if path.len() > 4096 {
            return Err(SecurityError::InvalidInput("Path too long".to_string()).into());
        }

        Ok(())
    }

    /// Check if an absolute path is in allowed directories
    fn is_allowed_absolute_path(path: &str) -> bool {
        let allowed_prefixes = [
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/opt",
            "/home",
            "/Users",
        ];

        allowed_prefixes.iter().any(|prefix| path.starts_with(prefix))
    }

    /// Validate command arguments for security
    pub fn validate_command_args(args: &[String]) -> ShellResult<()> {
        for arg in args {
            let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '<', '>', '"', '\''];
            if arg.chars().any(|c| dangerous_chars.contains(&c)) {
                return Err(SecurityError::DangerousCommand(
                    format!("Dangerous character in argument: {}", arg)
                ).into());
            }

            if arg.len() > 1024 {
                return Err(SecurityError::InvalidInput("Argument too long".to_string()).into());
            }
        }
        Ok(())
    }

    /// Sanitize user input by removing potentially dangerous characters
    pub fn sanitize_input(input: &str) -> String {
        input.chars()
            .filter(|&c| c.is_alphanumeric() || " .-_/".contains(c))
            .collect()
    }
}

/// Logging utilities for security events
pub mod logging {
    use std::fs::OpenOptions;
    use std::io::Write;
    use chrono::Utc;

    /// Log a security event
    pub fn log_security_event(event: &str, details: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("[{}] SECURITY: {} - {}\n", timestamp, event, details);

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("shell-t-security.log")
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        eprintln!("Security event: {} - {}", event, details);
    }

    /// Log a command execution for audit purposes
    pub fn log_command_execution(command: &str, user: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("[{}] AUDIT: User '{}' executed: {}\n", timestamp, user, command);

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("shell-t-audit.log")
        {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }
}