use std::io::{self, Write};
use std::process::Command;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;

mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Shell-T - Secure Multi-Language Terminal");
    println!("Type 'exit' to quit\n");

    loop {
        io::stdout()
            .execute(SetForegroundColor(Color::Green))?
            .execute(Print("shell-t> "))?
            .execute(ResetColor)?;
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            println!("Goodbye!");
            break;
        }

        match parser::parse_command(input) {
            Ok(commands) => {
                if let Err(e) = execute_commands(&commands) {
                    eprintln!("Error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
        }
    }

    Ok(())
}

fn execute_commands(commands: &[parser::Command]) -> Result<(), Box<dyn std::error::Error>> {
    if commands.is_empty() {
        return Ok(());
    }

    for cmd in commands {
        if cmd.program.is_empty() {
            continue;
        }

        match cmd.program.as_str() {
            "cd" => {
                if let Some(dir) = cmd.args.get(0) {
                    std::env::set_current_dir(dir)?;
                }
                continue;
            }
            "pwd" => {
                println!("{}", std::env::current_dir()?.display());
                continue;
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => {}
        }

        let mut command = Command::new(&cmd.program);
        command.args(&cmd.args);

        if let Some(input_file) = &cmd.input_redirect {
            let file = std::fs::File::open(input_file)?;
            command.stdin(file);
        }

        if let Some(output_file) = &cmd.output_redirect {
            let file = if cmd.append {
                std::fs::OpenOptions::new().create(true).append(true).open(output_file)?
            } else {
                std::fs::File::create(output_file)?
            };
            command.stdout(file);
        }

        let status = command.status()?;
        if !status.success() {
            eprintln!("Command failed with exit code: {}", status);
        }
    }

    Ok(())
}
