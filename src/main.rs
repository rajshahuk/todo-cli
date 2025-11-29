use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

const TODO_FILE: &str = "todo.txt";

#[derive(Parser)]
#[command(name = "todo-cli")]
#[command(about = "A command line todo list manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo item
    Add { description: String },
    /// List todo items
    List {
        /// Show all items including done items
        #[arg(long)]
        all: bool,
        /// Sort by priority
        #[arg(long)]
        pr: bool,
    },
    /// Mark a todo item as done
    Done { line_number: usize },
    /// Set or clear priority for a todo item
    Pr {
        priority: String,
        line_number: usize,
    },
}

#[derive(Debug, Clone)]
struct TodoItem {
    line_number: usize,
    priority: Option<char>,
    description: String,
    done_date: Option<String>,
}

impl TodoItem {
    fn parse(line: &str, line_number: usize) -> Self {
        let mut priority = None;
        let mut done_date = None;
        let mut description = line.to_string();

        // Check for priority at the start
        if line.starts_with('(') && line.len() > 3 && line.chars().nth(2) == Some(')') {
            let pri_char = line.chars().nth(1).unwrap();
            if pri_char.is_ascii_alphabetic() {
                priority = Some(pri_char.to_ascii_uppercase());
                description = line[4..].to_string();
            }
        }

        // Check for done date
        let words: Vec<&str> = line.split_whitespace().collect();
        for word in words {
            if word.starts_with("D:") {
                done_date = Some(word[2..].to_string());
                break;
            }
        }

        TodoItem {
            line_number,
            priority,
            description,
            done_date,
        }
    }

    fn is_done(&self) -> bool {
        self.done_date.is_some()
    }

    fn display(&self) {
        // Line number in cyan
        print!("{} ", self.line_number.to_string().cyan());

        // Priority in magenta
        if let Some(pri) = self.priority {
            print!("({}) ", pri.to_string().magenta());
        }

        // Find and display start date first
        let words: Vec<&str> = self.description.split_whitespace().collect();
        let start_date = words.iter().find(|w| w.starts_with("S:"));
        if let Some(date) = start_date {
            print!("{} ", date);
        }

        // Parse and display the description with colored metadata
        for (i, word) in words.iter().enumerate() {
            if word.starts_with("@") {
                print!("{}", word.green());
            } else if word.starts_with("P:") {
                print!("{}", word.yellow());
            } else if word.starts_with("T:") {
                print!("{}", word.bright_blue());
            } else if word.starts_with("S:") {
                // Skip start date as we already displayed it
                continue;
            } else if word.starts_with("D:") {
                print!("{}", word);
            } else if word.starts_with("(") && word.len() > 2 && word.chars().nth(2) == Some(')') {
                // Skip priority marker if it appears in description
                if i == 0 && self.priority.is_some() {
                    continue;
                }
                print!("{}", word);
            } else {
                print!("{}", word);
            }

            if i < words.len() - 1 {
                print!(" ");
            }
        }
        println!();
    }
}

fn check_and_create_file() -> io::Result<()> {
    if !Path::new(TODO_FILE).exists() {
        let current_dir = std::env::current_dir()?;
        println!(
            "The file '{}' does not exist in {}",
            TODO_FILE,
            current_dir.display()
        );
        print!("Would you like to create it? (Y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_uppercase() == "Y" {
            File::create(TODO_FILE)?;
            println!("Created '{}' in {}", TODO_FILE, current_dir.display());
        } else {
            println!("File not created. Exiting.");
            std::process::exit(0);
        }
    }
    Ok(())
}

fn read_todos() -> io::Result<Vec<TodoItem>> {
    let file = File::open(TODO_FILE)?;
    let reader = BufReader::new(file);
    let mut todos = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if !line.trim().is_empty() {
            todos.push(TodoItem::parse(&line, index + 1));
        }
    }

    Ok(todos)
}

fn write_todos(todos: &[String]) -> io::Result<()> {
    let mut file = File::create(TODO_FILE)?;
    for todo in todos {
        writeln!(file, "{}", todo)?;
    }
    Ok(())
}

fn add_todo(description: &str) -> io::Result<()> {
    check_and_create_file()?;

    let today = Local::now().format("%Y/%m/%d").to_string();
    let todo_line = format!("{} S:{}", description, today);

    let mut file = OpenOptions::new()
        .append(true)
        .open(TODO_FILE)?;

    writeln!(file, "{}", todo_line)?;
    println!("Added todo item");
    Ok(())
}

fn list_todos(show_all: bool, sort_by_priority: bool) -> io::Result<()> {
    check_and_create_file()?;

    let mut todos = read_todos()?;

    // Filter out done items unless --all is specified
    if !show_all {
        todos.retain(|todo| !todo.is_done());
    }

    if todos.is_empty() {
        println!("No todo items found");
        return Ok(());
    }

    // Sort by priority if requested
    if sort_by_priority {
        todos.sort_by(|a, b| {
            match (a.priority, b.priority) {
                (Some(p1), Some(p2)) => p1.cmp(&p2),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.line_number.cmp(&b.line_number),
            }
        });
    }

    for todo in todos {
        todo.display();
    }

    Ok(())
}

fn mark_done(line_number: usize) -> io::Result<()> {
    check_and_create_file()?;

    let content = fs::read_to_string(TODO_FILE)?;
    let lines: Vec<&str> = content.lines().collect();

    if line_number == 0 || line_number > lines.len() {
        eprintln!("Error: Todo item {} does not exist", line_number);
        return Ok(());
    }

    let target_line = lines[line_number - 1];
    let todo = TodoItem::parse(target_line, line_number);

    if todo.is_done() {
        eprintln!("Error: Todo item {} is already marked as done", line_number);
        return Ok(());
    }

    // Display confirmation
    println!("Mark this item as done?");
    print!("  {}\n", target_line);
    print!("(Y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_uppercase() != "Y" {
        println!("Cancelled");
        return Ok(());
    }

    // Add done date
    let today = Local::now().format("%Y/%m/%d").to_string();
    let updated_line = format!("{} D:{}", target_line, today);

    let mut new_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == line_number - 1 {
            new_lines.push(updated_line.clone());
        } else {
            new_lines.push(line.to_string());
        }
    }

    write_todos(&new_lines)?;
    println!("Todo item {} marked as done", line_number);
    Ok(())
}

fn set_priority(priority_str: &str, line_number: usize) -> io::Result<()> {
    check_and_create_file()?;

    let content = fs::read_to_string(TODO_FILE)?;
    let lines: Vec<&str> = content.lines().collect();

    if line_number == 0 || line_number > lines.len() {
        eprintln!("Error: Todo item {} does not exist", line_number);
        return Ok(());
    }

    let target_line = lines[line_number - 1];

    let updated_line = if priority_str.to_lowercase() == "clear" {
        // Remove priority
        if target_line.starts_with('(') && target_line.len() > 3 && target_line.chars().nth(2) == Some(')') {
            target_line[4..].to_string()
        } else {
            target_line.to_string()
        }
    } else {
        // Validate priority
        if priority_str.len() != 1 {
            eprintln!("Error: Priority must be a single character (A-Z)");
            return Ok(());
        }

        let pri_char = priority_str.chars().next().unwrap().to_ascii_uppercase();
        if !pri_char.is_ascii_alphabetic() {
            eprintln!("Error: Priority must be a letter (A-Z)");
            return Ok(());
        }

        // Remove existing priority if present
        let base_line = if target_line.starts_with('(') && target_line.len() > 3 && target_line.chars().nth(2) == Some(')') {
            target_line[4..].to_string()
        } else {
            target_line.to_string()
        };

        format!("({}) {}", pri_char, base_line)
    };

    let mut new_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == line_number - 1 {
            new_lines.push(updated_line.clone());
        } else {
            new_lines.push(line.to_string());
        }
    }

    write_todos(&new_lines)?;
    if priority_str.to_lowercase() == "clear" {
        println!("Cleared priority for todo item {}", line_number);
    } else {
        println!("Set priority for todo item {}", line_number);
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add { description } => add_todo(&description),
        Commands::List { all, pr } => list_todos(all, pr),
        Commands::Done { line_number } => mark_done(line_number),
        Commands::Pr { priority, line_number } => set_priority(&priority, line_number),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_todo() {
        let line = "Buy milk S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.line_number, 1);
        assert_eq!(todo.priority, None);
        assert_eq!(todo.description, "Buy milk S:2025/11/29");
        assert_eq!(todo.done_date, None);
    }

    #[test]
    fn test_parse_todo_with_priority() {
        let line = "(A) Buy milk S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.line_number, 1);
        assert_eq!(todo.priority, Some('A'));
        assert_eq!(todo.description, "Buy milk S:2025/11/29");
        assert_eq!(todo.done_date, None);
    }

    #[test]
    fn test_parse_todo_with_lowercase_priority() {
        let line = "(b) Buy milk S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.priority, Some('B'));
    }

    #[test]
    fn test_parse_todo_with_metadata() {
        let line = "Buy milk @shopping P:Personal T:urgent S:2025/11/29";
        let todo = TodoItem::parse(line, 5);

        assert_eq!(todo.line_number, 5);
        assert!(todo.description.contains("@shopping"));
        assert!(todo.description.contains("P:Personal"));
        assert!(todo.description.contains("T:urgent"));
    }

    #[test]
    fn test_parse_done_todo() {
        let line = "Buy milk S:2025/11/29 D:2025/11/30";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.done_date, Some("2025/11/30".to_string()));
        assert!(todo.is_done());
    }

    #[test]
    fn test_parse_done_todo_with_priority() {
        let line = "(A) Buy milk S:2025/11/29 D:2025/11/30";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.priority, Some('A'));
        assert_eq!(todo.done_date, Some("2025/11/30".to_string()));
        assert!(todo.is_done());
    }

    #[test]
    fn test_is_done_false() {
        let line = "Buy milk S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert!(!todo.is_done());
    }

    #[test]
    fn test_priority_uppercase_conversion() {
        let line = "(z) Low priority task S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.priority, Some('Z'));
    }

    #[test]
    fn test_parse_multiple_tags() {
        let line = "Review code T:review T:backend T:urgent S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert!(todo.description.contains("T:review"));
        assert!(todo.description.contains("T:backend"));
        assert!(todo.description.contains("T:urgent"));
    }

    #[test]
    fn test_parse_without_priority_marker() {
        let line = "Regular task without priority S:2025/11/29";
        let todo = TodoItem::parse(line, 1);

        assert_eq!(todo.priority, None);
        assert_eq!(todo.description, "Regular task without priority S:2025/11/29");
    }

    #[test]
    fn test_parse_complex_todo() {
        let line = "(B) Send email about meeting @work P:ProjectX T:urgent T:important S:2025/11/29";
        let todo = TodoItem::parse(line, 3);

        assert_eq!(todo.line_number, 3);
        assert_eq!(todo.priority, Some('B'));
        assert!(todo.description.contains("@work"));
        assert!(todo.description.contains("P:ProjectX"));
        assert!(todo.description.contains("T:urgent"));
        assert!(todo.description.contains("T:important"));
        assert_eq!(todo.done_date, None);
    }
}
