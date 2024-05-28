use chrono::Local;
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use dirs::home_dir;
use regex::escape;
use regex::Regex;
use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser)]
#[command(name = "zhistory")]
#[command(version = "1.0")]
#[command(about = "Command line history tool", long_about = None)]
struct Cli {
    // number of lines to read
    #[arg(short, long)]
    lines: Option<usize>,

    // string to search
    #[arg(short, long)]
    search: Option<String>,

    // take history upto a day
    #[arg(long, default_value = "false")]
    day: bool,

    // take history upto a month
    #[arg(long, default_value = "false")]
    month: bool,

    // print unique commands
    #[arg(long, default_value = "false")]
    stats: bool,
}

fn tokenize_and_filter(filter: &str, words: &mut Vec<&mut String>) {
    let tokens: Vec<String> = filter
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    words.retain(|word| {
        let lower_word = word.to_lowercase();
        tokens.iter().any(|token| lower_word.contains(token))
    });
}

fn main() -> io::Result<()> {
    let args: Cli = Cli::parse();
    // Ensure that either day or month is true, but not both
    if args.day && args.month {
        eprintln!("Error: Either --day or --month must be true, but not both.");
        std::process::exit(1);
    }
    if args.stats && args.search.is_some() {
        eprintln!("Error: --stats cannot be used with --search.");
        std::process::exit(1);
    }
    // Open the file in read-only mode
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
    let lines_to_read: usize = args.lines.unwrap_or(usize::MAX);
    println!("Reading the last {} lines from history file", lines_to_read);
    let mut line_number = 1;
    let search = args.search.clone().unwrap_or("".to_string()); // Clone the search value
    let mut lines: Vec<String> = reader
        .lines()
        .filter_map(|line_result| {
            process_line(line_result, &args, Some(search.clone()), &mut line_number)
            // Pass the cloned search value
        })
        .collect();

    // Get the specified number of lines from the end
    let num_lines: usize = min(lines_to_read, lines.len());
    let start_index: usize = lines.len().saturating_sub(num_lines);
    let last_lines: &mut [String] = &mut lines[start_index..];

    last_lines.reverse();
    if args.stats {
        let unique_commands: Vec<(String, usize)> = process_cmds(last_lines.to_vec(), ';');
        let top_cmds = 10;
        println!("Top {} most used commands: ", top_cmds);
        for (cmd, count) in unique_commands.iter().take(top_cmds) {
            println!("{}: {}", cmd, count);
        }
    } else {
        let mut i = 0;
        let mut last_lines: Vec<_> = last_lines.into_iter().collect();
        // Print the lines
        for line in &last_lines {
            println!("{}: {}", i, line);
            i += 1;
        }

        loop {
            // take filter input
            let mut filter_query = String::new();
            io::stdin()
                .read_line(&mut filter_query)
                .expect("Failed to read line");

            println!("Query received: {}", filter_query);
            if filter_query.trim().to_lowercase() == "q"
                || filter_query.trim().to_lowercase() == "exit"
                || filter_query.trim().to_lowercase() == "quit"
            {
                break;
            }

            if let Ok(parsed_int) = filter_query.trim().parse::<i32>() {
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
                    eprintln!("❌ Error: command not found in the input string");
                }
                break;
            } else {
                // user filters from the list of commands
                tokenize_and_filter(&filter_query, &mut last_lines);

                println!("{} Relevant results found:", last_lines.len());
                for (index, line) in last_lines.iter().enumerate() {
                    println!("{}: {}", index, line);
                }
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
            // eprintln!("Error: Unable to parse timestamp");
        }
    } else {
        // eprintln!("Error: Timestamp not found in the input string");
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

fn extract_unique_commands(line: &str, delimiter: char, unique_cmds: &mut HashMap<String, usize>) {
    let split_line: Vec<&str> = line.split(delimiter).collect();
    let cmd = split_line[1];
    let cmds: Vec<&str> = cmd.split_whitespace().take(2).collect();
    for cmd in cmds {
        if let Some(count) = unique_cmds.get_mut(cmd) {
            *count += 1;
        } else {
            unique_cmds.insert(cmd.to_string(), 1);
        }
    }
}

fn process_cmds(line_result: Vec<String>, delimiter: char) -> Vec<(String, usize)> {
    let mut unique_cmds: HashMap<String, usize> = HashMap::new();
    for line in line_result {
        extract_unique_commands(&line, delimiter, &mut unique_cmds);
    }
    let mut sorted_sequences: Vec<(String, usize)> = unique_cmds.into_iter().collect();
    sorted_sequences.sort_by(|a, b| b.1.cmp(&a.1));
    sorted_sequences
}

fn process_line(
    line_result: Result<String, io::Error>,
    args: &Cli,
    search: Option<String>,
    line_number: &mut usize,
) -> Option<String> {
    match line_result {
        Ok(bytes) => {
            *line_number += 1;
            let line = String::from_utf8_lossy(bytes.as_bytes()).into_owned();
            let s = Some(search);
            if match_regex(&line, s.as_ref().unwrap()) {
                if let Some(timestamp) = parse_timestamp(&line) {
                    // Parse the timestamp string to an integer
                    if (args.day && is_within_one_day(timestamp))
                        || (args.month && is_within_one_day(timestamp))
                    {
                        return Some(line);
                    } else if args.month && is_within_one_month(timestamp) {
                        return Some(line);
                    } else if !args.day && !args.month {
                        return Some(line);
                    }
                } else {
                    // eprintln!("Error: Timestamp not found in the input string");
                    return None;
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
