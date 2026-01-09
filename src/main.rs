use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

const TODO_FILE: &str = "todo.json";

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
        /// Filter by age (e.g., +1d for older than 1 day, +2w for 2 weeks, +3m for 3 months, +1y for 1 year)
        age_filter: Option<String>,
        /// Hide items marked as waiting (@WF)
        #[arg(long)]
        hide_waiting: bool,
    },
    /// Mark a todo item as done
    Done { line_number: usize },
    /// Edit a todo item
    Edit { line_number: usize },
    /// Set or clear priority for a todo item
    Pr {
        priority: String,
        line_number: usize,
    },
    /// List all unique projects
    Projects,
    /// Convert a todo.txt file to todo.json format
    Convert {
        /// Path to the input todo.txt file
        input: String,
        /// Path to the output JSON file (defaults to todo.json)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    #[serde(skip)]
    line_number: usize,
    priority: Option<char>,
    description: String,
    context: Option<String>,
    project: Option<String>,
    tags: Vec<String>,
    start_date: String,
    done_date: Option<String>,
    #[serde(default)]
    due_date: Option<String>,
}

// Parse user input to extract metadata
fn parse_metadata(
    input: &str,
) -> (
    String,
    Option<String>,
    Option<String>,
    Vec<String>,
    Option<String>,
) {
    let mut description_words = Vec::new();
    let mut context = None;
    let mut project = None;
    let mut tags = Vec::new();
    let mut due_date = None;

    for word in input.split_whitespace() {
        if let Some(stripped) = word.strip_prefix("@") {
            if context.is_none() {
                context = Some(stripped.to_string());
            }
            // Skip all @ words, not just the first
        } else if word.starts_with("P:") || word.starts_with("p:") {
            if project.is_none() {
                project = Some(word[2..].to_string());
            }
            // Skip all P: words, not just the first
        } else if word.starts_with("T:") || word.starts_with("t:") {
            tags.push(word[2..].to_string());
        } else if word.starts_with("Due:") || word.starts_with("due:") {
            if due_date.is_none() {
                let date_str = &word[4..];
                due_date = parse_due_date_input(date_str);
            }
        } else {
            description_words.push(word);
        }
    }

    let description = description_words.join(" ");
    (description, context, project, tags, due_date)
}

impl TodoItem {
    fn is_done(&self) -> bool {
        self.done_date.is_some()
    }

    fn is_overdue(&self) -> bool {
        if let Some(due) = &self.due_date {
            let today = Local::now().format("%Y/%m/%d").to_string();
            due < &today
        } else {
            false
        }
    }

    fn display(&self) {
        // Line number in cyan
        print!("{} ", self.line_number.to_string().cyan());

        // Priority in magenta
        if let Some(pri) = self.priority {
            print!("({}) ", pri.to_string().magenta());
        }

        // Start date
        print!("S:{} ", self.start_date);

        // Due date - show after start date, before description
        if let Some(due) = &self.due_date {
            if self.is_overdue() {
                print!("Due:{} ", due.red().bold()); // Overdue in RED and BOLD
            } else {
                print!("Due:{} ", due); // Normal display
            }
        }

        // Description
        print!("{} ", self.description);

        // Context
        if let Some(ctx) = &self.context {
            print!("@{} ", ctx.green());
        }

        // Project
        if let Some(proj) = &self.project {
            print!("P:{} ", proj.yellow());
        }

        // Tags
        for tag in &self.tags {
            print!("T:{} ", tag.bright_blue());
        }

        // Done date
        if let Some(done) = &self.done_date {
            print!("D:{} ", done);
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
    let content = fs::read_to_string(TODO_FILE)?;

    let mut todos: Vec<TodoItem> = serde_json::from_str(&content).unwrap_or_else(|_| Vec::new());

    // Assign line numbers based on array index
    for (i, todo) in todos.iter_mut().enumerate() {
        todo.line_number = i + 1;
    }

    Ok(todos)
}

fn write_todos(todos: &[TodoItem]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(todos).map_err(io::Error::other)?;
    fs::write(TODO_FILE, json)?;
    Ok(())
}

// Parse age filter string (e.g., "+1d", "+2w", "+3m", "+1y")
// Returns (value, unit) where unit is 'd', 'w', 'm', or 'y'
fn parse_age_filter(filter: &str) -> Option<(i64, char)> {
    let trimmed = filter.trim();

    // Must start with '+'
    if !trimmed.starts_with('+') {
        return None;
    }

    let without_plus = &trimmed[1..];

    // Must have at least 2 characters (number + unit)
    if without_plus.len() < 2 {
        return None;
    }

    // Extract the unit (last character)
    let unit = without_plus.chars().last()?;

    // Validate unit
    if !matches!(unit, 'd' | 'w' | 'm' | 'y') {
        return None;
    }

    // Extract and parse the number
    let number_str = &without_plus[..without_plus.len() - 1];
    let value = number_str.parse::<i64>().ok()?;

    // Value must be positive
    if value <= 0 {
        return None;
    }

    Some((value, unit))
}

// Calculate cutoff date based on age filter
// Returns a date string in "YYYY/MM/DD" format
fn calculate_cutoff_date(value: i64, unit: char) -> String {
    use chrono::Duration;

    let now = Local::now();
    let cutoff = match unit {
        'd' => now - Duration::days(value),
        'w' => now - Duration::weeks(value),
        'm' => now - Duration::days(value * 30), // Approximate month as 30 days
        'y' => now - Duration::days(value * 365), // Approximate year as 365 days
        _ => now,                                // Should never happen due to validation
    };

    cutoff.format("%Y/%m/%d").to_string()
}

// Calculate a future date based on duration (inverse of calculate_cutoff_date)
fn calculate_future_date(value: i64, unit: char) -> String {
    use chrono::Duration;

    let now = Local::now();
    let future = match unit {
        'd' => now + Duration::days(value),
        'w' => now + Duration::weeks(value),
        'm' => now + Duration::days(value * 30), // Approximate month as 30 days
        'y' => now + Duration::days(value * 365), // Approximate year as 365 days
        _ => now,
    };

    future.format("%Y/%m/%d").to_string()
}

// Validate date string format (basic check)
// Expected format: YYYY/MM/DD
fn validate_date_format(date_str: &str) -> bool {
    let parts: Vec<&str> = date_str.split('/').collect();

    if parts.len() != 3 {
        return false;
    }

    // Check year (4 digits)
    if parts[0].len() != 4 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check month (2 digits, 01-12)
    if parts[1].len() != 2 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let month: u32 = parts[1].parse().unwrap_or(0);
    if !(1..=12).contains(&month) {
        return false;
    }

    // Check day (2 digits, 01-31)
    if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let day: u32 = parts[2].parse().unwrap_or(0);
    if !(1..=31).contains(&day) {
        return false;
    }

    true
}

// Parse due date input - handles both absolute dates and relative dates
// Absolute: "2025-12-25" or "2025/12/25"
// Relative: "+3d", "+2w", "+1m"
// Returns: Option<String> in YYYY/MM/DD format, or None if invalid
fn parse_due_date_input(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Check if it's a relative date (starts with '+')
    if trimmed.starts_with('+') {
        // Parse like age filter: +3d, +2w, +1m
        if let Some((value, unit)) = parse_age_filter(trimmed) {
            // Calculate future date instead of past date
            return Some(calculate_future_date(value, unit));
        }
        return None;
    }

    // Handle absolute date - accept both YYYY-MM-DD and YYYY/MM/DD
    let normalized = trimmed.replace('-', "/");

    // Basic validation: check format YYYY/MM/DD
    if validate_date_format(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

fn add_todo(description: &str) -> io::Result<()> {
    check_and_create_file()?;

    let mut todos = read_todos()?;

    // Parse metadata from description
    let (clean_desc, context, project, tags, due_date) = parse_metadata(description);

    let new_item = TodoItem {
        line_number: todos.len() + 1,
        priority: None,
        description: clean_desc,
        context,
        project,
        tags,
        start_date: Local::now().format("%Y/%m/%d").to_string(),
        done_date: None,
        due_date,
    };

    todos.push(new_item);
    write_todos(&todos)?;
    println!("Added todo item");
    Ok(())
}

fn list_todos(
    show_all: bool,
    sort_by_priority: bool,
    age_filter: Option<String>,
    hide_waiting: bool,
) -> io::Result<()> {
    check_and_create_file()?;

    let mut todos = read_todos()?;

    // Filter out done items unless --all is specified
    if !show_all {
        todos.retain(|todo| !todo.is_done());
    }

    // Apply age filter if provided
    if let Some(filter) = age_filter {
        match parse_age_filter(&filter) {
            Some((value, unit)) => {
                let cutoff_date = calculate_cutoff_date(value, unit);
                todos.retain(|todo| {
                    // Compare start_date with cutoff_date
                    // A todo is "older than" the age if its start_date <= cutoff_date
                    todo.start_date <= cutoff_date
                });
            }
            None => {
                eprintln!(
                    "Error: Invalid age filter format. Use format like +1d, +2w, +3m, or +1y"
                );
                eprintln!("  d = days, w = weeks, m = months, y = years");
                return Ok(());
            }
        }
    }

    // Filter out waiting items if --hide-waiting is specified
    if hide_waiting {
        todos.retain(|todo| {
            if let Some(context) = &todo.context {
                context.to_uppercase() != "WF"
            } else {
                true
            }
        });
    }

    if todos.is_empty() {
        println!("No todo items found");
        return Ok(());
    }

    // Sort todos with smart prioritization:
    // 1. Items with BOTH due date AND priority (sorted by priority, then by due date)
    // 2. Items with due date only (sorted by due date)
    // 3. Items with priority only (sorted by priority)
    // 4. Items with neither (sorted by line number)
    todos.sort_by(|a, b| {
        use std::cmp::Ordering;

        match (&a.due_date, &a.priority, &b.due_date, &b.priority) {
            // Both items have due date AND priority
            (Some(due_a), Some(pri_a), Some(due_b), Some(pri_b)) => {
                // First compare by priority, then by due date
                match pri_a.cmp(pri_b) {
                    Ordering::Equal => due_a.cmp(due_b),
                    other => other,
                }
            }
            // a has both, b doesn't - a comes first
            (Some(_), Some(_), _, _) => Ordering::Less,
            // b has both, a doesn't - b comes first
            (_, _, Some(_), Some(_)) => Ordering::Greater,

            // Both have due date but no priority
            (Some(due_a), None, Some(due_b), None) => due_a.cmp(due_b),
            // a has due date only, b has priority only - a comes first
            (Some(_), None, None, Some(_)) => Ordering::Less,
            // a has due date only, b has neither - a comes first
            (Some(_), None, None, None) => Ordering::Less,
            // b has due date only, a has priority only - b comes first
            (None, Some(_), Some(_), None) => Ordering::Greater,
            // b has due date only, a has neither - b comes first
            (None, None, Some(_), None) => Ordering::Greater,

            // Both have priority but no due date
            (None, Some(pri_a), None, Some(pri_b)) => pri_a.cmp(pri_b),
            // a has priority only, b has neither - a comes first
            (None, Some(_), None, None) => Ordering::Less,
            // b has priority only, a has neither - b comes first
            (None, None, None, Some(_)) => Ordering::Greater,

            // Neither has due date or priority
            (None, None, None, None) => a.line_number.cmp(&b.line_number),
        }
    });

    // If --pr flag is used, apply additional priority sorting (legacy behavior)
    if sort_by_priority {
        // The --pr flag now just forces priority sorting for items without due dates
        // Items with due dates are already optimally sorted above
        todos.sort_by(|a, b| {
            use std::cmp::Ordering;

            // Keep items with due dates in their current order
            match (&a.due_date, &b.due_date) {
                (Some(_), Some(_)) => Ordering::Equal, // Preserve order
                (Some(_), None) => Ordering::Less,     // Items with due dates stay first
                (None, Some(_)) => Ordering::Greater,
                (None, None) => {
                    // For items without due dates, sort by priority
                    match (a.priority, b.priority) {
                        (Some(p1), Some(p2)) => p1.cmp(&p2),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => Ordering::Equal,
                    }
                }
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

    let mut todos = read_todos()?;

    if line_number == 0 || line_number > todos.len() {
        eprintln!("Error: Todo item {} does not exist", line_number);
        return Ok(());
    }

    let todo = &todos[line_number - 1];

    if todo.is_done() {
        eprintln!("Error: Todo item {} is already marked as done", line_number);
        return Ok(());
    }

    // Display confirmation - show formatted todo item
    println!("Mark this item as done?");
    print!("  ");
    if let Some(pri) = todo.priority {
        print!("({}) ", pri);
    }
    print!("{}", todo.description);
    if let Some(ctx) = &todo.context {
        print!(" @{}", ctx);
    }
    if let Some(proj) = &todo.project {
        print!(" P:{}", proj);
    }
    for tag in &todo.tags {
        print!(" T:{}", tag);
    }
    if let Some(due) = &todo.due_date {
        print!(" Due:{}", due);
    }
    println!(" S:{}", todo.start_date);
    print!("(Y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_uppercase() != "Y" {
        println!("Cancelled");
        return Ok(());
    }

    // Add done date
    todos[line_number - 1].done_date = Some(Local::now().format("%Y/%m/%d").to_string());

    write_todos(&todos)?;
    println!("Todo item {} marked as done", line_number);
    Ok(())
}

fn set_priority(priority_str: &str, line_number: usize) -> io::Result<()> {
    check_and_create_file()?;

    let mut todos = read_todos()?;

    if line_number == 0 || line_number > todos.len() {
        eprintln!("Error: Todo item {} does not exist", line_number);
        return Ok(());
    }

    if priority_str.to_lowercase() == "clear" {
        // Remove priority
        todos[line_number - 1].priority = None;
        write_todos(&todos)?;
        println!("Cleared priority for todo item {}", line_number);
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

        // Set priority
        todos[line_number - 1].priority = Some(pri_char);
        write_todos(&todos)?;
        println!("Set priority for todo item {}", line_number);
    }

    Ok(())
}

// Helper function to read input with a default value shown
// If user presses Enter without typing, returns None (keep current value)
// If user types something, returns Some(value)
fn read_input_with_default(prompt: &str, current_value: &str) -> io::Result<Option<String>> {
    print!("{} [{}]: ", prompt, current_value);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        Ok(None) // Keep current value
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn edit_todo(line_number: usize) -> io::Result<()> {
    check_and_create_file()?;

    let mut todos = read_todos()?;

    if line_number == 0 || line_number > todos.len() {
        eprintln!("Error: Todo item {} does not exist", line_number);
        return Ok(());
    }

    let todo = &todos[line_number - 1];

    println!("Editing todo item {}:", line_number);
    println!("Press Enter to keep current value, or type new value\n");

    // Edit description
    let current_desc = &todo.description;
    let new_description = read_input_with_default("Description", current_desc)?;

    // Edit priority
    let current_priority = todo
        .priority
        .map(|c| c.to_string())
        .unwrap_or_else(|| "none".to_string());
    let new_priority = read_input_with_default("Priority (A-Z, or 'clear')", &current_priority)?;

    // Edit context
    let current_context = todo.context.as_deref().unwrap_or("none");
    let new_context = read_input_with_default("Context (without @)", current_context)?;

    // Edit project
    let current_project = todo.project.as_deref().unwrap_or("none");
    let new_project = read_input_with_default("Project (without P:)", current_project)?;

    // Edit tags
    let current_tags = if todo.tags.is_empty() {
        "none".to_string()
    } else {
        todo.tags.join(", ")
    };
    let new_tags = read_input_with_default("Tags (comma-separated, without T:)", &current_tags)?;

    // Edit due date
    let current_due = todo.due_date.as_deref().unwrap_or("none");
    let new_due_date =
        read_input_with_default("Due date (YYYY-MM-DD, +3d, +2w, or 'clear')", current_due)?;

    // Apply changes
    let todo_mut = &mut todos[line_number - 1];

    if let Some(desc) = new_description {
        todo_mut.description = desc;
    }

    if let Some(pri) = new_priority {
        if pri.to_lowercase() == "clear" || pri.to_lowercase() == "none" {
            todo_mut.priority = None;
        } else if pri.len() == 1 {
            let pri_char = pri.chars().next().unwrap().to_ascii_uppercase();
            if pri_char.is_ascii_alphabetic() {
                todo_mut.priority = Some(pri_char);
            } else {
                eprintln!("Warning: Invalid priority '{}', keeping current value", pri);
            }
        } else {
            eprintln!("Warning: Invalid priority '{}', keeping current value", pri);
        }
    }

    if let Some(ctx) = new_context {
        if ctx.to_lowercase() == "clear" || ctx.to_lowercase() == "none" {
            todo_mut.context = None;
        } else {
            todo_mut.context = Some(ctx);
        }
    }

    if let Some(proj) = new_project {
        if proj.to_lowercase() == "clear" || proj.to_lowercase() == "none" {
            todo_mut.project = None;
        } else {
            todo_mut.project = Some(proj);
        }
    }

    if let Some(tags_str) = new_tags {
        if tags_str.to_lowercase() == "clear" || tags_str.to_lowercase() == "none" {
            todo_mut.tags = Vec::new();
        } else {
            todo_mut.tags = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    if let Some(due_str) = new_due_date {
        if due_str.to_lowercase() == "clear" || due_str.to_lowercase() == "none" {
            todo_mut.due_date = None;
        } else if let Some(parsed_date) = parse_due_date_input(&due_str) {
            todo_mut.due_date = Some(parsed_date);
        } else {
            eprintln!(
                "Warning: Invalid due date format '{}', keeping current value",
                due_str
            );
            eprintln!("Expected format: YYYY-MM-DD or +3d, +2w, +1m, +1y");
        }
    }

    write_todos(&todos)?;
    println!("\nTodo item {} updated successfully", line_number);

    Ok(())
}

fn parse_txt_line(line: &str) -> TodoItem {
    let mut priority = None;
    let mut context = None;
    let mut project = None;
    let mut tags = Vec::new();
    let mut start_date = String::new();
    let mut done_date = None;
    let mut due_date = None;
    let mut description_words = Vec::new();

    let trimmed = line.trim();
    let mut remaining = trimmed;

    // Check for priority at the start: (A) format
    if remaining.starts_with('(') && remaining.len() > 3 && remaining.chars().nth(2) == Some(')') {
        let pri_char = remaining.chars().nth(1).unwrap();
        if pri_char.is_ascii_alphabetic() {
            priority = Some(pri_char.to_ascii_uppercase());
            remaining = remaining[4..].trim_start();
        }
    }

    // Parse the rest of the line word by word
    for word in remaining.split_whitespace() {
        if word.starts_with("@") && word.len() > 1 {
            if context.is_none() {
                context = Some(word[1..].to_string());
            }
        } else if (word.starts_with("P:") || word.starts_with("p:")) && word.len() > 2 {
            if project.is_none() {
                project = Some(word[2..].to_string());
            }
        } else if (word.starts_with("T:") || word.starts_with("t:")) && word.len() > 2 {
            tags.push(word[2..].to_string());
        } else if (word.starts_with("S:") || word.starts_with("s:")) && word.len() > 2 {
            start_date = word[2..].to_string();
        } else if (word.starts_with("D:") || word.starts_with("d:")) && word.len() > 2 {
            done_date = Some(word[2..].to_string());
        } else if (word.starts_with("Due:") || word.starts_with("due:")) && word.len() > 4 {
            if due_date.is_none() {
                due_date = Some(word[4..].to_string());
            }
        } else {
            description_words.push(word);
        }
    }

    TodoItem {
        line_number: 0,
        priority,
        description: description_words.join(" "),
        context,
        project,
        tags,
        start_date,
        done_date,
        due_date,
    }
}

fn convert_file(input: &str, output: Option<String>) -> io::Result<()> {
    let output_path = output.unwrap_or_else(|| TODO_FILE.to_string());

    // Check if input file exists
    if !Path::new(input).exists() {
        eprintln!("Error: Input file '{}' does not exist", input);
        std::process::exit(1);
    }

    // Check if output file exists and prompt for overwrite
    if Path::new(&output_path).exists() {
        print!(
            "Output file '{}' already exists. Overwrite? (Y/N): ",
            output_path
        );
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if response.trim().to_uppercase() != "Y" {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Read and parse the txt file
    let content = fs::read_to_string(input)?;
    let mut todos: Vec<TodoItem> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            todos.push(parse_txt_line(trimmed));
        }
    }

    // Write to JSON
    let json = serde_json::to_string_pretty(&todos).map_err(io::Error::other)?;
    fs::write(&output_path, json)?;

    println!(
        "Converted {} todo items from '{}' to '{}'",
        todos.len(),
        input,
        output_path
    );
    Ok(())
}

fn list_projects() -> io::Result<()> {
    check_and_create_file()?;

    let todos = read_todos()?;

    // Collect unique projects
    let mut projects: Vec<String> = todos
        .iter()
        .filter_map(|todo| todo.project.clone())
        .collect();

    // Remove duplicates and sort
    projects.sort();
    projects.dedup();

    if projects.is_empty() {
        println!("No projects found");
        return Ok(());
    }

    println!("Projects:");
    for project in projects {
        println!("  P:{}", project.yellow());
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add { description } => add_todo(&description),
        Commands::List {
            all,
            pr,
            age_filter,
            hide_waiting,
        } => list_todos(all, pr, age_filter, hide_waiting),
        Commands::Done { line_number } => mark_done(line_number),
        Commands::Edit { line_number } => edit_todo(line_number),
        Commands::Pr {
            priority,
            line_number,
        } => set_priority(&priority, line_number),
        Commands::Projects => list_projects(),
        Commands::Convert { input, output } => convert_file(&input, output),
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
    fn test_parse_metadata_simple() {
        let input = "Buy milk";
        let (desc, context, project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(context, None);
        assert_eq!(project, None);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_metadata_with_context() {
        let input = "Buy milk @shopping";
        let (desc, context, project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(context, Some("shopping".to_string()));
        assert_eq!(project, None);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_metadata_with_project() {
        let input = "Buy milk P:Personal";
        let (desc, context, project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(context, None);
        assert_eq!(project, Some("Personal".to_string()));
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_metadata_with_tags() {
        let input = "Review code T:urgent T:backend";
        let (desc, context, project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Review code");
        assert_eq!(context, None);
        assert_eq!(project, None);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "urgent");
        assert_eq!(tags[1], "backend");
    }

    #[test]
    fn test_parse_metadata_complex() {
        let input = "Send email about meeting @work P:ProjectX T:urgent T:important";
        let (desc, context, project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Send email about meeting");
        assert_eq!(context, Some("work".to_string()));
        assert_eq!(project, Some("ProjectX".to_string()));
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "urgent");
        assert_eq!(tags[1], "important");
    }

    #[test]
    fn test_parse_metadata_first_context_only() {
        let input = "Task @first @second";
        let (desc, context, _project, _tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Task");
        assert_eq!(context, Some("first".to_string()));
    }

    #[test]
    fn test_parse_metadata_first_project_only() {
        let input = "Task P:First P:Second";
        let (desc, _context, project, _tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Task");
        assert_eq!(project, Some("First".to_string()));
    }

    #[test]
    fn test_parse_metadata_lowercase_project() {
        let input = "Buy milk p:Personal";
        let (desc, _context, project, _tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(project, Some("Personal".to_string()));
    }

    #[test]
    fn test_parse_metadata_lowercase_tags() {
        let input = "Fix bug t:urgent t:backend";
        let (desc, _context, _project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Fix bug");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "urgent");
        assert_eq!(tags[1], "backend");
    }

    #[test]
    fn test_parse_metadata_mixed_case() {
        let input = "Task p:Project1 T:tag1 t:tag2 P:Project2";
        let (desc, _context, project, tags, _due_date) = parse_metadata(input);

        assert_eq!(desc, "Task");
        assert_eq!(project, Some("Project1".to_string())); // First one wins
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "tag1");
        assert_eq!(tags[1], "tag2");
    }

    #[test]
    fn test_todo_item_is_done() {
        let todo = TodoItem {
            line_number: 1,
            priority: None,
            description: "Buy milk".to_string(),
            context: None,
            project: None,
            tags: Vec::new(),
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/11/30".to_string()),
            due_date: None,
        };

        assert!(todo.is_done());
    }

    #[test]
    fn test_todo_item_is_not_done() {
        let todo = TodoItem {
            line_number: 1,
            priority: None,
            description: "Buy milk".to_string(),
            context: None,
            project: None,
            tags: Vec::new(),
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        };

        assert!(!todo.is_done());
    }

    #[test]
    fn test_todo_item_serialization() {
        let todo = TodoItem {
            line_number: 1,
            priority: Some('A'),
            description: "Buy milk".to_string(),
            context: Some("shopping".to_string()),
            project: Some("Personal".to_string()),
            tags: vec!["urgent".to_string()],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        };

        let json = serde_json::to_string(&todo).unwrap();
        assert!(json.contains("Buy milk"));
        assert!(json.contains("shopping"));
        assert!(json.contains("Personal"));
        assert!(json.contains("urgent"));
        assert!(!json.contains("line_number"));
    }

    #[test]
    fn test_todo_item_deserialization() {
        let json = r#"{
            "priority": "A",
            "description": "Buy milk",
            "context": "shopping",
            "project": "Personal",
            "tags": ["urgent"],
            "start_date": "2025/11/29",
            "done_date": null
        }"#;

        let todo: TodoItem = serde_json::from_str(json).unwrap();
        assert_eq!(todo.priority, Some('A'));
        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.context, Some("shopping".to_string()));
        assert_eq!(todo.project, Some("Personal".to_string()));
        assert_eq!(todo.tags.len(), 1);
        assert_eq!(todo.start_date, "2025/11/29");
        assert_eq!(todo.done_date, None);
    }

    // Tests for parse_txt_line (convert command)

    #[test]
    fn test_parse_txt_line_simple() {
        let line = "Buy milk S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.priority, None);
        assert_eq!(todo.context, None);
        assert_eq!(todo.project, None);
        assert!(todo.tags.is_empty());
        assert_eq!(todo.start_date, "2025/11/29");
        assert_eq!(todo.done_date, None);
    }

    #[test]
    fn test_parse_txt_line_with_priority() {
        let line = "(A) Buy milk S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.priority, Some('A'));
        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.start_date, "2025/11/29");
    }

    #[test]
    fn test_parse_txt_line_lowercase_priority() {
        let line = "(b) Call dentist S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.priority, Some('B'));
        assert_eq!(todo.description, "Call dentist");
    }

    #[test]
    fn test_parse_txt_line_with_context() {
        let line = "Buy milk @shopping S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.context, Some("shopping".to_string()));
    }

    #[test]
    fn test_parse_txt_line_with_project() {
        let line = "Buy milk P:Personal S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.project, Some("Personal".to_string()));
    }

    #[test]
    fn test_parse_txt_line_with_tags() {
        let line = "Review code T:urgent T:backend S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Review code");
        assert_eq!(todo.tags.len(), 2);
        assert_eq!(todo.tags[0], "urgent");
        assert_eq!(todo.tags[1], "backend");
    }

    #[test]
    fn test_parse_txt_line_with_done_date() {
        let line = "Buy milk S:2025/11/29 D:2025/11/30";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.start_date, "2025/11/29");
        assert_eq!(todo.done_date, Some("2025/11/30".to_string()));
    }

    #[test]
    fn test_parse_txt_line_complex() {
        let line =
            "(B) Send email about meeting @work P:ProjectX T:urgent T:important S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.priority, Some('B'));
        assert_eq!(todo.description, "Send email about meeting");
        assert_eq!(todo.context, Some("work".to_string()));
        assert_eq!(todo.project, Some("ProjectX".to_string()));
        assert_eq!(todo.tags.len(), 2);
        assert_eq!(todo.tags[0], "urgent");
        assert_eq!(todo.tags[1], "important");
        assert_eq!(todo.start_date, "2025/11/29");
        assert_eq!(todo.done_date, None);
    }

    #[test]
    fn test_parse_txt_line_first_context_only() {
        let line = "Task @first @second S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Task");
        assert_eq!(todo.context, Some("first".to_string()));
    }

    #[test]
    fn test_parse_txt_line_first_project_only() {
        let line = "Task P:First P:Second S:2025/11/29";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Task");
        assert_eq!(todo.project, Some("First".to_string()));
    }

    #[test]
    fn test_parse_txt_line_lowercase_markers() {
        let line = "Task @home p:personal t:urgent s:2025/11/29 d:2025/11/30";
        let todo = parse_txt_line(line);

        assert_eq!(todo.description, "Task");
        assert_eq!(todo.context, Some("home".to_string()));
        assert_eq!(todo.project, Some("personal".to_string()));
        assert_eq!(todo.tags, vec!["urgent"]);
        assert_eq!(todo.start_date, "2025/11/29");
        assert_eq!(todo.done_date, Some("2025/11/30".to_string()));
    }

    #[test]
    fn test_parse_txt_line_done_with_priority() {
        let line = "(A) Completed task @work S:2025/11/28 D:2025/11/30";
        let todo = parse_txt_line(line);

        assert_eq!(todo.priority, Some('A'));
        assert_eq!(todo.description, "Completed task");
        assert_eq!(todo.context, Some("work".to_string()));
        assert_eq!(todo.start_date, "2025/11/28");
        assert_eq!(todo.done_date, Some("2025/11/30".to_string()));
    }

    #[test]
    fn test_parse_txt_line_whitespace_handling() {
        let line = "  (A) Buy milk @shopping S:2025/11/29  ";
        let todo = parse_txt_line(line);

        assert_eq!(todo.priority, Some('A'));
        assert_eq!(todo.description, "Buy milk");
        assert_eq!(todo.context, Some("shopping".to_string()));
    }

    // Tests for age filter functionality

    #[test]
    fn test_parse_age_filter_days() {
        let result = parse_age_filter("+1d");
        assert_eq!(result, Some((1, 'd')));

        let result = parse_age_filter("+7d");
        assert_eq!(result, Some((7, 'd')));
    }

    #[test]
    fn test_parse_age_filter_weeks() {
        let result = parse_age_filter("+2w");
        assert_eq!(result, Some((2, 'w')));
    }

    #[test]
    fn test_parse_age_filter_months() {
        let result = parse_age_filter("+3m");
        assert_eq!(result, Some((3, 'm')));
    }

    #[test]
    fn test_parse_age_filter_years() {
        let result = parse_age_filter("+1y");
        assert_eq!(result, Some((1, 'y')));
    }

    #[test]
    fn test_parse_age_filter_invalid_no_plus() {
        let result = parse_age_filter("1d");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_age_filter_invalid_unit() {
        let result = parse_age_filter("+1x");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_age_filter_invalid_no_number() {
        let result = parse_age_filter("+d");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_age_filter_invalid_negative() {
        let result = parse_age_filter("+-1d");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_age_filter_invalid_zero() {
        let result = parse_age_filter("+0d");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_age_filter_with_whitespace() {
        let result = parse_age_filter(" +5d ");
        assert_eq!(result, Some((5, 'd')));
    }

    #[test]
    fn test_calculate_cutoff_date_format() {
        let cutoff = calculate_cutoff_date(1, 'd');
        // Check that the format matches YYYY/MM/DD
        assert!(cutoff.len() == 10);
        assert!(cutoff.contains('/'));
        let parts: Vec<&str> = cutoff.split('/').collect();
        assert_eq!(parts.len(), 3);
        // Year should be 4 digits
        assert_eq!(parts[0].len(), 4);
        // Month and day should be 2 digits
        assert_eq!(parts[1].len(), 2);
        assert_eq!(parts[2].len(), 2);
    }
}
