use clap::Parser;
use dirs::home_dir;
use std::cmp::min;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser)]
#[command(name = "greeter")]
#[command(version = "1.0")]
#[command(about = "Says hello to a person", long_about = None)]
struct Cli {
    // number of lines to read
    #[arg(short, long, default_value = "5")]
    lines: usize,

    // string to search
    #[arg(short, long)]
    search: Option<String>,
}

fn main() -> io::Result<()> {
    let args: Cli = Cli::parse();
    // Open the file in read-only mode
    println!("Reading the last {} lines from history file", args.lines);
    let history_file_path: PathBuf = match get_history_file_path() {
        Some(path) => path,
        None => {
            eprintln!("Could not determine history file path.");
            return Ok(());
        }
    };

    let file: File = File::open(&history_file_path)?;

    // Create a buffered reader
    let reader: BufReader<File> = BufReader::new(file);
    let mut line_number = 1;
    let mut lines: Vec<String> = reader
        .lines()
        .filter_map(|line: Result<String, io::Error>| match line {
            Ok(bytes) => {
                line_number += 1;
                Some(String::from_utf8_lossy(bytes.as_bytes()).into_owned())
            }
            Err(e) => {
                eprintln!("Error reading line number {}: {}", line_number, e);
                line_number += 1;
                None
            }
        })
        .collect();

    // Get the specified number of lines from the end
    let num_lines: usize = min(args.lines, lines.len());
    let start_index: usize = lines.len().saturating_sub(num_lines);
    let last_lines: &mut [String] = &mut lines[start_index..];

    last_lines.reverse();

    let mut i = 0;

    // Print the lines
    for line in last_lines {
        println!("{}: {}", i, line);
        i += 1;
    }

    Ok(())
}

fn get_history_file_path() -> Option<std::path::PathBuf> {
    let home_dir = home_dir()?;
    let shell = env::var("SHELL").ok()?;

    if shell.contains("zsh") {
        Some(home_dir.join(".zsh_history"))
    } else if shell.contains("bash") {
        Some(home_dir.join(".bash_history"))
    } else {
        println!("Unsupported shell: {}", shell);
        None
    }
}
