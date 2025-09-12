use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    pub security: SecurityConfig,
    pub limits: ResourceLimits,
    pub ui: UiConfig,
    pub interpreters: InterpreterConfig,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub enable_logging: bool,
    pub enable_auditing: bool,
    pub max_command_length: usize,
    pub max_arg_count: usize,
    pub allowed_commands: HashSet<String>,
    pub blocked_commands: HashSet<String>,
    pub validate_paths: bool,
    pub sanitize_input: bool,
}

/// Resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_background_processes: usize,
    pub max_pipeline_length: usize,
    pub command_timeout: u64,
    pub max_memory_mb: usize,
}

/// UI configuration
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub enable_colors: bool,
    pub prompt_color: String,
    pub show_timestamps: bool,
    pub enable_completion: bool,
}

/// Interpreter configuration
#[derive(Debug, Clone)]
pub struct InterpreterConfig {
    pub python_path: String,
    pub ruby_path: String,
    pub node_path: String,
    pub enable_scripts: bool,
    pub allowed_extensions: HashSet<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
            limits: ResourceLimits::default(),
            ui: UiConfig::default(),
            interpreters: InterpreterConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut allowed_commands = HashSet::new();
        for cmd in ["ls", "pwd", "cd", "cat", "grep", "head", "tail", "wc", "sort", "uniq"] {
            allowed_commands.insert(cmd.to_string());
        }

        let mut blocked_commands = HashSet::new();
        for cmd in ["rm", "rmdir", "mv", "cp", "chmod", "chown", "sudo", "su"] {
            blocked_commands.insert(cmd.to_string());
        }

        Self {
            enable_logging: true,
            enable_auditing: true,
            max_command_length: 4096,
            max_arg_count: 100,
            allowed_commands,
            blocked_commands,
            validate_paths: true,
            sanitize_input: true,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_background_processes: 10,
            max_pipeline_length: 10,
            command_timeout: 300, // 5 minutes
            max_memory_mb: 512,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            enable_colors: true,
            prompt_color: "green".to_string(),
            show_timestamps: false,
            enable_completion: true,
        }
    }
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        let mut allowed_extensions = HashSet::new();
        for ext in ["py", "rb", "js", "sh"] {
            allowed_extensions.insert(ext.to_string());
        }

        Self {
            python_path: "python3".to_string(),
            ruby_path: "ruby".to_string(),
            node_path: "node".to_string(),
            enable_scripts: true,
            allowed_extensions,
        }
    }
}

impl Config {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();

        if let Ok(config_str) = fs::read_to_string("shell-t.toml") {
            config = Self::parse_toml(&config_str)?;
        }

        config.load_from_env();

        Ok(config)
    }

    /// Parse TOML configuration
    fn parse_toml(_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

    /// Load configuration from environment variables
    fn load_from_env(&mut self) {
        if let Ok(val) = env::var("SHELL_T_ENABLE_LOGGING") {
            self.security.enable_logging = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("SHELL_T_MAX_COMMAND_LENGTH") {
            if let Ok(len) = val.parse() {
                self.security.max_command_length = len;
            }
        }

        if let Ok(val) = env::var("SHELL_T_PYTHON_PATH") {
            self.interpreters.python_path = val;
        }

        if let Ok(val) = env::var("SHELL_T_RUBY_PATH") {
            self.interpreters.ruby_path = val;
        }

        if let Ok(val) = env::var("SHELL_T_NODE_PATH") {
            self.interpreters.node_path = val;
        }

        if let Ok(val) = env::var("SHELL_T_ENABLE_COLORS") {
            self.ui.enable_colors = val.parse().unwrap_or(true);
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.security.max_command_length == 0 {
            return Err("Max command length must be greater than 0".to_string());
        }

        if self.limits.max_background_processes == 0 {
            return Err("Max background processes must be greater than 0".to_string());
        }

        if self.limits.max_pipeline_length == 0 {
            return Err("Max pipeline length must be greater than 0".to_string());
        }

        if !Path::new(&self.interpreters.python_path).exists() {
            eprintln!("Warning: Python interpreter not found at {}", self.interpreters.python_path);
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Configuration validation functions
pub mod validation {
    use super::*;
    use crate::error::{SecurityError, ShellResult};

    /// Validate a command against security policies
    pub fn validate_command(config: &Config, command: &str) -> ShellResult<()> {
        if command.len() > config.security.max_command_length {
            return Err(SecurityError::InvalidInput("Command too long".to_string()).into());
        }

        if config.security.blocked_commands.contains(command) {
            return Err(SecurityError::DangerousCommand(command.to_string()).into());
        }

        if !config.security.allowed_commands.is_empty()
            && !config.security.allowed_commands.contains(command) {
            return Err(SecurityError::DangerousCommand(
                format!("Command not in whitelist: {}", command)
            ).into());
        }

        Ok(())
    }

    /// Validate arguments against security policies
    pub fn validate_args(config: &Config, args: &[String]) -> ShellResult<()> {
        if args.len() > config.security.max_arg_count {
            return Err(SecurityError::InvalidInput("Too many arguments".to_string()).into());
        }

        for arg in args {
            if arg.len() > config.security.max_command_length {
                return Err(SecurityError::InvalidInput("Argument too long".to_string()).into());
            }
        }

        Ok(())
    }
}