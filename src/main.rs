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
    },
    /// Mark a todo item as done
    Done { line_number: usize },
    /// Set or clear priority for a todo item
    Pr {
        priority: String,
        line_number: usize,
    },
    /// List all unique projects
    Projects,
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
}

// Parse user input to extract metadata
fn parse_metadata(input: &str) -> (String, Option<String>, Option<String>, Vec<String>) {
    let mut description_words = Vec::new();
    let mut context = None;
    let mut project = None;
    let mut tags = Vec::new();

    for word in input.split_whitespace() {
        if word.starts_with("@") {
            if context.is_none() {
                context = Some(word[1..].to_string());
            }
            // Skip all @ words, not just the first
        } else if word.starts_with("P:") || word.starts_with("p:") {
            if project.is_none() {
                project = Some(word[2..].to_string());
            }
            // Skip all P: words, not just the first
        } else if word.starts_with("T:") || word.starts_with("t:") {
            tags.push(word[2..].to_string());
        } else {
            description_words.push(word);
        }
    }

    let description = description_words.join(" ");
    (description, context, project, tags)
}

impl TodoItem {
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

        // Start date
        print!("S:{} ", self.start_date);

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

    let mut todos: Vec<TodoItem> = serde_json::from_str(&content)
        .unwrap_or_else(|_| Vec::new());

    // Assign line numbers based on array index
    for (i, todo) in todos.iter_mut().enumerate() {
        todo.line_number = i + 1;
    }

    Ok(todos)
}

fn write_todos(todos: &[TodoItem]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(todos)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(TODO_FILE, json)?;
    Ok(())
}

fn add_todo(description: &str) -> io::Result<()> {
    check_and_create_file()?;

    let mut todos = read_todos()?;

    // Parse metadata from description
    let (clean_desc, context, project, tags) = parse_metadata(description);

    let new_item = TodoItem {
        line_number: todos.len() + 1,
        priority: None,
        description: clean_desc,
        context,
        project,
        tags,
        start_date: Local::now().format("%Y/%m/%d").to_string(),
        done_date: None,
    };

    todos.push(new_item);
    write_todos(&todos)?;
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
    print!(" S:{}\n", todo.start_date);
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
        Commands::List { all, pr } => list_todos(all, pr),
        Commands::Done { line_number } => mark_done(line_number),
        Commands::Pr { priority, line_number } => set_priority(&priority, line_number),
        Commands::Projects => list_projects(),
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
        let (desc, context, project, tags) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(context, None);
        assert_eq!(project, None);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_metadata_with_context() {
        let input = "Buy milk @shopping";
        let (desc, context, project, tags) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(context, Some("shopping".to_string()));
        assert_eq!(project, None);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_metadata_with_project() {
        let input = "Buy milk P:Personal";
        let (desc, context, project, tags) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(context, None);
        assert_eq!(project, Some("Personal".to_string()));
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_parse_metadata_with_tags() {
        let input = "Review code T:urgent T:backend";
        let (desc, context, project, tags) = parse_metadata(input);

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
        let (desc, context, project, tags) = parse_metadata(input);

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
        let (desc, context, _project, _tags) = parse_metadata(input);

        assert_eq!(desc, "Task");
        assert_eq!(context, Some("first".to_string()));
    }

    #[test]
    fn test_parse_metadata_first_project_only() {
        let input = "Task P:First P:Second";
        let (desc, _context, project, _tags) = parse_metadata(input);

        assert_eq!(desc, "Task");
        assert_eq!(project, Some("First".to_string()));
    }

    #[test]
    fn test_parse_metadata_lowercase_project() {
        let input = "Buy milk p:Personal";
        let (desc, _context, project, _tags) = parse_metadata(input);

        assert_eq!(desc, "Buy milk");
        assert_eq!(project, Some("Personal".to_string()));
    }

    #[test]
    fn test_parse_metadata_lowercase_tags() {
        let input = "Fix bug t:urgent t:backend";
        let (desc, _context, _project, tags) = parse_metadata(input);

        assert_eq!(desc, "Fix bug");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "urgent");
        assert_eq!(tags[1], "backend");
    }

    #[test]
    fn test_parse_metadata_mixed_case() {
        let input = "Task p:Project1 T:tag1 t:tag2 P:Project2";
        let (desc, _context, project, tags) = parse_metadata(input);

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
}
