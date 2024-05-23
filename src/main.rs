use chrono::{Local, TimeZone};
use clap::Parser;
use dirs::home_dir;
use regex::escape;
use regex::Regex;
use std::cmp::min;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser)]
#[command(name = "zshhistory")]
#[command(version = "1.0")]
#[command(about = "Command line history tool", long_about = None)]
struct Cli {
    // number of lines to read
    #[arg(short, long, default_value = "5")]
    lines: usize,

    // string to search
    #[arg(short, long, default_value = "")]
    search: Option<String>,

    // take history upto a day
    #[arg(long, default_value = "false")]
    day: bool,

    // take history upto a month
    #[arg(long, default_value = "false")]
    month: bool,
}

// struct HistoryItem {
//     command: String,
//     timestamp: i64,
// }

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
        .filter_map(|line_result| match line_result {
            Ok(bytes) => {
                line_number += 1;
                let line = String::from_utf8_lossy(bytes.as_bytes()).into_owned();
                let escaped_s = match args.search.as_ref() {
                    Some(s) => escape(s.as_str()),
                    None => {
                        eprintln!("Error: search argument is missing");
                        return None;
                    }
                };
                let pattern = format!("\\b{}\\b", escaped_s);
                let regex = match Regex::new(&pattern) {
                    Ok(re) => re,
                    Err(e) => {
                        eprintln!("Error: Invalid regex pattern: {}", e);
                        return None;
                    }
                };

                if let Some(..) = regex.captures(&line) {
                    let parts: Vec<&str> = line.split(':').collect();
                    if let Some(timestamp_str) = parts.get(1) {
                        // Parse the timestamp string to an integer
                        if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
                            if (args.day && is_within_one_day(timestamp))
                                || (args.month && is_within_one_day(timestamp))
                            {
                                println!("Timestamp: {}", timestamp);
                                return Some(line);
                            } else if args.month && is_within_one_month(timestamp) {
                                println!("Timestamp: {}", timestamp);
                                return Some(line);
                            } else if !args.day && !args.month {
                                return Some(line);
                            }
                        } else {
                            eprintln!("Error: Unable to parse timestamp");
                        }
                    } else {
                        eprintln!("Error: Timestamp not found in the input string");
                    }

                    Some(line)
                } else {
                    None
                }
            }
            Err(..) => {
                // eprintln!("Error reading line number {}: {}", line_number, e);
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

// Check if the timestamp is within a day
fn is_within_one_day(timestamp: i64) -> bool {
    let current_time = Local::now().timestamp();
    let one_day_ago = current_time - 24 * 3600;
    timestamp >= one_day_ago
}

// Check if the timestamp is within a month
fn is_within_one_month(timestamp: i64) -> bool {
    let current_time = Local::now().timestamp();
    let one_month_ago = current_time - 30 * 24 * 3600;
    let one_month_from_now = current_time + 30 * 24 * 3600;
    timestamp >= one_month_ago && timestamp <= one_month_from_now
}
