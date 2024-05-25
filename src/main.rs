use chrono::Local;
use clap::Parser;
use dirs::home_dir;
use regex::escape;
use regex::Regex;
use std::cmp::min;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::string;
use std::process::Command;
use clipboard::{ClipboardContext, ClipboardProvider};


/// Simple program to greet a person
#[derive(Parser)]
#[command(name = "zhistory")]
#[command(version = "1.0")]
#[command(about = "Command line history tool", long_about = None)]
struct Cli {
    // number of lines to read
    #[arg(short, long, default_value = "100")]
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
        .filter_map(|line_result| process_line(line_result, &args, &mut line_number))
        .collect();

    // Get the specified number of lines from the end
    let num_lines: usize = min(args.lines, lines.len());
    let start_index: usize = lines.len().saturating_sub(num_lines);
    let last_lines: &mut [String] = &mut lines[start_index..];

    let mut last_lines: Vec<_> = last_lines
    .into_iter()
    .collect();

    last_lines.reverse();

    let mut i = 0;

    // Print the lines
    for line in &last_lines {
        println!("{}: {}", i, line);
        i += 1;
    }

    while(true){
        // take filter input
        let mut filterQuery = String::new();
        io::stdin()
            .read_line(&mut filterQuery)
            .expect("Failed to read line");

        println!("Query received: {}", filterQuery);
        if filterQuery.trim().to_lowercase() == "q" || filterQuery.trim().to_lowercase() == "exit" || filterQuery.trim().to_lowercase() == "quit" {
            break;
        }

        if let Ok(parsed_int) = filterQuery.trim().parse::<i32>() {
            // user selects a command to run
            let parsed_uint: usize = parsed_int as usize;
            let entire_line = last_lines[parsed_uint].clone();
            let parts: Vec<&str> = entire_line.split(';').collect();

            if let Some(commmand_str) = parts.get(1) {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(commmand_str.to_string()).unwrap();
                println!("📋 Copied into your clipboard!");
                break;
            } else {
                eprintln!("Error: command not found in the input string");
            }
            break;
        } else {
            // user filters from the list of commands
            let re = Regex::new(r"(?i)Pass").unwrap(); // Replace Pass with filterQuery
            // filter commands
            last_lines.retain(|line| re.is_match(line));
            
            println!("{} Relevant results found:", last_lines.len());
            for (index, line) in last_lines.iter().enumerate() {
                println!("{}: {}", index, line);
            }
        }

        
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

// Function to parse timestamp from a line
fn parse_timestamp(line: &str) -> Option<i64> {
    let parts: Vec<&str> = line.split(':').collect();
    if let Some(timestamp_str) = parts.get(1) {
        if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
            return Some(timestamp);
        } else {
            eprintln!("Error: Unable to parse timestamp");
        }
    } else {
        eprintln!("Error: Timestamp not found in the input string");
    }
    None
}

// Function to match regex pattern on a line
fn match_regex(line: &str, search: &Option<String>) -> bool {
    if let Some(search_str) = search {
        let escaped_s = escape(search_str);
        let pattern = format!("\\b{}\\b", escaped_s);
        if let Ok(regex) = Regex::new(&pattern) {
            if regex.captures(line).is_some() {
                return true;
            }
        } else {
            eprintln!("Error: Invalid regex pattern");
        }
    }
    false
}

fn process_line(
    line_result: Result<String, io::Error>,
    args: &Cli,
    line_number: &mut usize,
) -> Option<String> {
    match line_result {
        Ok(bytes) => {
            *line_number += 1;
            let line = String::from_utf8_lossy(bytes.as_bytes()).into_owned();

            if match_regex(&line, &args.search) {
                if let Some(timestamp) = parse_timestamp(&line) {
                    // Parse the timestamp string to an integer
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
                    eprintln!("Error: Timestamp not found in the input string");
                }

                Some(line)
            } else {
                None
            }
        }
        Err(..) => {
            // eprintln!("Error reading line number {}: {}", line_number, e);
            *line_number += 1;
            None
        }
    }
}
