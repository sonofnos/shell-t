# Shell-T

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance terminal shell built in Rust with multi-language script execution support.

## Features

- **Multi-Language Support**: Execute Python, Ruby, and JavaScript files directly
- **Command Pipelines**: Chain commands with `|` operator
- **I/O Redirection**: Input/output redirection with `<`, `>`, `>>`
- **Background Jobs**: Run commands asynchronously with `&`
- **Built-in Commands**: `cd`, `pwd`, `exit`, `help`
- **Colored Interface**: Enhanced terminal experience

## Installation

### Build from Source

```bash
# Prerequisites: Rust 1.70+
git clone <repository-url>
cd shell-t
cargo build --release
```

## Usage

```bash
# Start the shell
./target/release/shell-t

# Basic commands
pwd                    # Print working directory
cd <directory>         # Change directory
ls                     # List files
exit                   # Exit shell

# Multi-language execution
./script.py           # Run Python script
./script.rb           # Run Ruby script
./script.js           # Run JavaScript file

# Pipelines and redirection
ls | grep txt         # Pipeline
echo "hello" > file.txt  # Output redirection
sort < input.txt      # Input redirection

# Background jobs
sleep 10 &            # Run in background
```

## License

MIT License
