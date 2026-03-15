use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::sync::Mutex;

const TEST_TODO_FILE: &str = "todo.json";
const TEST_PROJECTS_FILE: &str = "projects.json";

// Global lock to ensure tests run serially
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectInfo {
    name: String,
    status: String,
    last_reviewed: Option<String>,
}

fn setup() {
    // Remove test files if they exist
    let _ = fs::remove_file(TEST_TODO_FILE);
    let _ = fs::remove_file(TEST_PROJECTS_FILE);
}

fn teardown() {
    // Clean up test files
    let _ = fs::remove_file(TEST_TODO_FILE);
    let _ = fs::remove_file(TEST_PROJECTS_FILE);
}

fn get_binary_path() -> std::path::PathBuf {
    // Use cargo's built-in test binary path
    // This works across all platforms and test scenarios
    std::env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path.push(if cfg!(windows) {
                "todo-cli.exe"
            } else {
                "todo-cli"
            });
            path
        })
        .unwrap_or_else(|| {
            // Fallback to the old method if env path doesn't work
            let binary_name = if cfg!(windows) {
                "todo-cli.exe"
            } else {
                "todo-cli"
            };
            std::path::PathBuf::from(format!("./target/debug/{}", binary_name))
        })
}

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(get_binary_path())
        .args(args)
        .output()
        .expect("Failed to execute command")
}

fn run_command_with_input(args: &[&str], input: &str) -> std::process::Output {
    use std::io::Write;
    let mut child = Command::new(get_binary_path())
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    child
        .wait_with_output()
        .expect("Failed to wait for command")
}

fn create_test_file_with_todos(todos: Vec<TodoItem>) {
    let json = serde_json::to_string_pretty(&todos).expect("Failed to serialize todos");
    fs::write(TEST_TODO_FILE, json).expect("Failed to write test file");
}

fn create_test_projects_file(projects: Vec<ProjectInfo>) {
    let json = serde_json::to_string_pretty(&projects).expect("Failed to serialize projects");
    fs::write(TEST_PROJECTS_FILE, json).expect("Failed to write test projects file");
}

fn make_project(name: &str, status: &str, last_reviewed: Option<&str>) -> ProjectInfo {
    ProjectInfo {
        name: name.to_string(),
        status: status.to_string(),
        last_reviewed: last_reviewed.map(|s| s.to_string()),
    }
}

fn make_todo(description: &str, priority: Option<char>, done_date: Option<&str>) -> TodoItem {
    TodoItem {
        priority,
        description: description.to_string(),
        context: None,
        project: None,
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: done_date.map(|s| s.to_string()),
        due_date: None,
    }
}

#[test]
fn test_add_simple_todo() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Create file first
    run_command_with_input(&["add", "Buy milk"], "Y\n");

    // Verify file exists and contains the todo
    let content = fs::read_to_string(TEST_TODO_FILE);
    if content.is_err() {
        teardown();
        panic!("Failed to read test file");
    }

    let content = content.unwrap();
    assert!(content.contains("Buy milk"));
    assert!(content.contains("start_date"));

    teardown();
}

#[test]
fn test_add_todo_with_metadata() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command_with_input(&["add", "Buy milk @shopping P:Personal T:urgent"], "Y\n");

    let content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(content.contains("Buy milk"));
    assert!(content.contains("shopping"));
    assert!(content.contains("Personal"));
    assert!(content.contains("urgent"));

    teardown();
}

#[test]
fn test_list_empty() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();
    create_test_file_with_todos(vec![]);

    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No todo items found") || stdout.is_empty());

    teardown();
}

#[test]
fn test_list_filters_done_items() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Buy milk".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Send email".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/11/30".to_string()),
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Buy milk"));
    assert!(!stdout.contains("Send email"));

    teardown();
}

#[test]
fn test_list_all_shows_done_items() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Buy milk".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Send email".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/11/30".to_string()),
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command(&["list", "--all"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Buy milk"));
    assert!(stdout.contains("Send email"));

    teardown();
}

#[test]
fn test_list_priority_sorting() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: Some('C'),
            description: "Task C".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: Some('A'),
            description: "Task A".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: Some('B'),
            description: "Task B".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command(&["list", "--pr"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify all tasks are present
    assert!(stdout.contains("Task A"));
    assert!(stdout.contains("Task B"));
    assert!(stdout.contains("Task C"));

    // Find positions of each task
    let pos_a = stdout.find("Task A").unwrap();
    let pos_b = stdout.find("Task B").unwrap();
    let pos_c = stdout.find("Task C").unwrap();

    // Verify they're in priority order
    assert!(pos_a < pos_b);
    assert!(pos_b < pos_c);

    teardown();
}

#[test]
fn test_set_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command(&["pr", "a", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Set priority"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"A\""));
    assert!(updated_content.contains("Buy milk"));

    teardown();
}

#[test]
fn test_change_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", Some('A'), None)]);

    run_command(&["pr", "b", "1"]);

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"B\""));
    assert!(!updated_content.contains("\"A\""));

    teardown();
}

#[test]
fn test_clear_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", Some('A'), None)]);

    let output = run_command(&["pr", "clear", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Cleared priority"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("null"));
    assert!(updated_content.contains("Buy milk"));

    teardown();
}

#[test]
fn test_mark_done() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command_with_input(&["done", "1"], "Y\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("marked as done"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("done_date"));

    teardown();
}

#[test]
fn test_mark_done_skip_confirm() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    // No stdin needed with --yes flag
    let output = run_command(&["done", "--yes", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("marked as done"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("done_date"));

    teardown();
}

#[test]
fn test_mark_done_skip_confirm_short_flag() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command(&["done", "-y", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("marked as done"));

    teardown();
}

#[test]
fn test_mark_done_cancelled() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command_with_input(&["done", "1"], "N\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Cancelled"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"done_date\": null"));

    teardown();
}

#[test]
fn test_mark_done_already_done() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, Some("2025/11/30"))]);

    let output = run_command_with_input(&["done", "1"], "Y\n");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("already marked as done"));

    teardown();
}

#[test]
fn test_mark_done_invalid_number() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command(&["done", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("does not exist"));

    teardown();
}

#[test]
fn test_priority_invalid_number() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command(&["pr", "a", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("does not exist"));

    teardown();
}

#[test]
fn test_lowercase_priority_converted() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    run_command(&["pr", "c", "1"]);

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"C\""));

    teardown();
}

#[test]
fn test_list_shows_line_numbers() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![
        make_todo("Task 1", None, None),
        make_todo("Task 2", None, None),
        make_todo("Task 3", None, None),
    ]);

    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
    assert!(stdout.contains("3"));

    teardown();
}

#[test]
fn test_priority_with_done_item() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", Some('A'), Some("2025/11/30"))]);

    let output = run_command(&["list", "--all"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("(A)"));
    assert!(stdout.contains("Buy milk"));

    teardown();
}

#[test]
fn test_projects_empty() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No projects found"));

    teardown();
}

#[test]
fn test_projects_single() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todo = TodoItem {
        priority: None,
        description: "Task 1".to_string(),
        context: None,
        project: Some("Backend".to_string()),
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    };

    create_test_file_with_todos(vec![todo]);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Projects:"));
    assert!(stdout.contains("P:Backend"));

    teardown();
}

#[test]
fn test_projects_multiple_unique() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Task 1".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 3".to_string(),
            context: None,
            project: Some("API".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];

    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Projects:"));
    assert!(stdout.contains("P:Backend"));
    assert!(stdout.contains("P:Frontend"));
    assert!(stdout.contains("P:API"));

    // Verify alphabetical order
    let api_pos = stdout.find("P:API").unwrap();
    let backend_pos = stdout.find("P:Backend").unwrap();
    let frontend_pos = stdout.find("P:Frontend").unwrap();
    assert!(api_pos < backend_pos);
    assert!(backend_pos < frontend_pos);

    teardown();
}

#[test]
fn test_projects_with_duplicates() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Task 1".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 3".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];

    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Projects:"));

    // Count occurrences of "P:Backend" - should only appear once
    let backend_count = stdout.matches("P:Backend").count();
    assert_eq!(backend_count, 1);

    teardown();
}

#[test]
fn test_projects_includes_done_items() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Task 1".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/11/30".to_string()),
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];

    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Projects:"));
    assert!(stdout.contains("P:Backend"));
    assert!(stdout.contains("P:Frontend"));

    teardown();
}

// Convert command tests

const TEST_TXT_FILE: &str = "test_todo.txt";
const TEST_OUTPUT_FILE: &str = "test_output.json";

fn setup_convert() {
    let _ = fs::remove_file(TEST_TXT_FILE);
    let _ = fs::remove_file(TEST_OUTPUT_FILE);
}

fn teardown_convert() {
    let _ = fs::remove_file(TEST_TXT_FILE);
    let _ = fs::remove_file(TEST_OUTPUT_FILE);
}

fn create_test_txt_file(content: &str) {
    fs::write(TEST_TXT_FILE, content).expect("Failed to write test txt file");
}

#[test]
fn test_convert_simple() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("Buy milk S:2025/11/29\n");

    let output = run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Converted 1 todo items"));

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("Buy milk"));
    assert!(json_content.contains("2025/11/29"));

    teardown_convert();
}

#[test]
fn test_convert_with_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("(A) Important task S:2025/11/29\n");

    run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("\"priority\": \"A\""));
    assert!(json_content.contains("Important task"));

    teardown_convert();
}

#[test]
fn test_convert_with_metadata() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("Buy milk @shopping P:Personal T:urgent S:2025/11/29\n");

    run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("Buy milk"));
    assert!(json_content.contains("\"context\": \"shopping\""));
    assert!(json_content.contains("\"project\": \"Personal\""));
    assert!(json_content.contains("urgent"));

    teardown_convert();
}

#[test]
fn test_convert_with_done_date() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("Completed task S:2025/11/28 D:2025/11/29\n");

    run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("Completed task"));
    assert!(json_content.contains("\"start_date\": \"2025/11/28\""));
    assert!(json_content.contains("\"done_date\": \"2025/11/29\""));

    teardown_convert();
}

#[test]
fn test_convert_multiple_items() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    let content = "Buy milk @shopping S:2025/11/29\n\
                   (A) Send email @work P:ProjectX T:urgent S:2025/11/28\n\
                   (B) Call dentist S:2025/11/27 D:2025/11/30\n";
    create_test_txt_file(content);

    let output = run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Converted 3 todo items"));

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("Buy milk"));
    assert!(json_content.contains("Send email"));
    assert!(json_content.contains("Call dentist"));

    teardown_convert();
}

#[test]
fn test_convert_missing_input_file() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    let output = run_command(&["convert", "nonexistent.txt", "-o", TEST_OUTPUT_FILE]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));

    teardown_convert();
}

#[test]
fn test_convert_overwrite_cancelled() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("Buy milk S:2025/11/29\n");
    fs::write(TEST_OUTPUT_FILE, "existing content").unwrap();

    let output = run_command_with_input(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE], "N\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Cancelled"));

    // Verify original content preserved
    let content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert_eq!(content, "existing content");

    teardown_convert();
}

#[test]
fn test_convert_overwrite_confirmed() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("Buy milk S:2025/11/29\n");
    fs::write(TEST_OUTPUT_FILE, "existing content").unwrap();

    let output = run_command_with_input(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE], "Y\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Converted 1 todo items"));

    // Verify content was overwritten
    let content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(content.contains("Buy milk"));

    teardown_convert();
}

#[test]
fn test_convert_empty_lines_skipped() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    let content = "Buy milk S:2025/11/29\n\n\nSend email S:2025/11/28\n\n";
    create_test_txt_file(content);

    let output = run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Converted 2 todo items"));

    teardown_convert();
}

#[test]
fn test_convert_multiple_tags() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("Review code T:urgent T:backend T:review S:2025/11/29\n");

    run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("urgent"));
    assert!(json_content.contains("backend"));
    assert!(json_content.contains("review"));

    teardown_convert();
}

#[test]
fn test_convert_lowercase_markers() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file("(b) Task @home p:personal t:quick s:2025/11/29 d:2025/11/30\n");

    run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("\"priority\": \"B\""));
    assert!(json_content.contains("\"context\": \"home\""));
    assert!(json_content.contains("\"project\": \"personal\""));
    assert!(json_content.contains("quick"));
    assert!(json_content.contains("\"start_date\": \"2025/11/29\""));
    assert!(json_content.contains("\"done_date\": \"2025/11/30\""));

    teardown_convert();
}

#[test]
fn test_convert_complex_description() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup_convert();

    create_test_txt_file(
        "(A) Send email about the meeting tomorrow @work P:ProjectX T:urgent T:important S:2025/11/29\n",
    );

    run_command(&["convert", TEST_TXT_FILE, "-o", TEST_OUTPUT_FILE]);

    let json_content = fs::read_to_string(TEST_OUTPUT_FILE).unwrap();
    assert!(json_content.contains("Send email about the meeting tomorrow"));

    teardown_convert();
}

// Edit command tests

#[test]
fn test_edit_description() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Original task", None, None)]);

    // Edit description: type new description, press Enter for all other fields
    let output = run_command_with_input(&["edit", "1"], "Updated task\n\n\n\n\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("Updated task"));
    assert!(!updated_content.contains("Original task"));

    teardown();
}

#[test]
fn test_edit_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    // Keep description, set priority to A, keep rest
    let output = run_command_with_input(&["edit", "1"], "\nA\n\n\n\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"priority\": \"A\""));
    assert!(updated_content.contains("Buy milk"));

    teardown();
}

#[test]
fn test_edit_context_and_project() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![TodoItem {
        priority: None,
        description: "Send email".to_string(),
        context: None,
        project: None,
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    }];
    create_test_file_with_todos(todos);

    // Keep description and priority, set context=work, project=Website, keep tags
    let output = run_command_with_input(&["edit", "1"], "\n\nwork\nWebsite\n\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"context\": \"work\""));
    assert!(updated_content.contains("\"project\": \"Website\""));
    assert!(updated_content.contains("Send email"));

    teardown();
}

#[test]
fn test_edit_tags() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Review code", None, None)]);

    // Keep all except tags, set tags to "urgent, important"
    let output = run_command_with_input(&["edit", "1"], "\n\n\n\nurgent, important\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("\"urgent\""));
    assert!(updated_content.contains("\"important\""));

    teardown();
}

#[test]
fn test_edit_clear_fields() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![TodoItem {
        priority: Some('A'),
        description: "Task with metadata".to_string(),
        context: Some("work".to_string()),
        project: Some("Project1".to_string()),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    }];
    create_test_file_with_todos(todos);

    // Keep description, clear priority, context, project, and tags
    let output = run_command_with_input(&["edit", "1"], "\nclear\nnone\nclear\nnone\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("Task with metadata"));
    assert!(updated_content.contains("\"priority\": null"));
    assert!(updated_content.contains("\"context\": null"));
    assert!(updated_content.contains("\"project\": null"));
    assert!(updated_content.contains("\"tags\": []"));

    teardown();
}

#[test]
fn test_edit_keep_current_values() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![TodoItem {
        priority: Some('B'),
        description: "Original description".to_string(),
        context: Some("home".to_string()),
        project: Some("Personal".to_string()),
        tags: vec!["test".to_string()],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    }];
    create_test_file_with_todos(todos);

    // Press Enter for all fields to keep current values
    let output = run_command_with_input(&["edit", "1"], "\n\n\n\n\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    // Content should be essentially the same (only formatting might differ)
    assert!(updated_content.contains("Original description"));
    assert!(updated_content.contains("\"B\""));
    assert!(updated_content.contains("home"));
    assert!(updated_content.contains("Personal"));
    assert!(updated_content.contains("test"));

    teardown();
}

#[test]
fn test_edit_invalid_number() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Task 1", None, None)]);

    let output = run_command_with_input(&["edit", "99"], "\n\n\n\n\n");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("does not exist"));

    teardown();
}

#[test]
fn test_edit_all_fields() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Old task", None, None)]);

    // Update all fields
    let output = run_command_with_input(
        &["edit", "1"],
        "New task\nC\noffice\nWorkProject\ntag1, tag2\n",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("New task"));
    assert!(updated_content.contains("\"C\""));
    assert!(updated_content.contains("office"));
    assert!(updated_content.contains("WorkProject"));
    assert!(updated_content.contains("tag1"));
    assert!(updated_content.contains("tag2"));
    assert!(!updated_content.contains("Old task"));

    teardown();
}

#[test]
fn test_add_todo_with_absolute_due_date() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command_with_input(&["add", "Task with due date Due:2026-06-15"], "Y\n");

    let content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(content.contains("Task with due date"));
    assert!(content.contains("2026/06/15"));
    assert!(content.contains("due_date"));

    teardown();
}

#[test]
fn test_add_todo_with_relative_due_date() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command_with_input(&["add", "Task due in 3 days Due:+3d"], "Y\n");

    let content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(content.contains("Task due in 3 days"));
    assert!(content.contains("due_date"));
    // The actual date will be calculated, so we just check it exists

    teardown();
}

#[test]
fn test_list_shows_due_dates() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command_with_input(&["add", "Task 1 Due:2026-01-10"], "Y\n");
    run_command_with_input(&["add", "Task 2 Due:2026-01-05"], "Y\n");
    run_command_with_input(&["add", "Task 3"], "Y\n");

    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that due dates are shown
    assert!(stdout.contains("Due:2026/01/05"));
    assert!(stdout.contains("Due:2026/01/10"));

    // Task 2 with earlier due date should appear before Task 1
    let task2_pos = stdout.find("Task 2").unwrap();
    let task1_pos = stdout.find("Task 1").unwrap();
    assert!(
        task2_pos < task1_pos,
        "Tasks should be sorted by due date (earliest first)"
    );

    teardown();
}

#[test]
fn test_edit_due_date() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Task to edit", None, None)]);

    // Edit and set a due date
    let output = run_command_with_input(&["edit", "1"], "\n\n\n\n\n2026-07-15\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("2026/07/15"));
    assert!(updated_content.contains("due_date"));

    teardown();
}

#[test]
fn test_edit_clear_due_date() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // First create a todo with a due date
    run_command_with_input(&["add", "Task with due Due:2026-08-20"], "Y\n");

    // Edit and clear the due date
    let output = run_command_with_input(&["edit", "1"], "\n\n\n\n\nclear\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("updated successfully"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    let todos: Vec<TodoItem> = serde_json::from_str(&updated_content).unwrap();
    assert!(todos[0].due_date.is_none());

    teardown();
}

#[test]
fn test_list_hide_waiting() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Add tasks with and without @WF context
    run_command_with_input(&["add", "Active task"], "Y\n");
    run_command_with_input(&["add", "Waiting task @WF"], "Y\n");
    run_command_with_input(&["add", "Another active @work"], "Y\n");

    // List without --hide-waiting should show all tasks
    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Active task"));
    assert!(stdout.contains("Waiting task"));
    assert!(stdout.contains("Another active"));

    // List with --hide-waiting should filter out @WF tasks
    let output = run_command(&["list", "--hide-waiting"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Active task"));
    assert!(!stdout.contains("Waiting task"));
    assert!(stdout.contains("Another active"));

    teardown();
}

#[test]
fn test_list_hide_waiting_case_insensitive() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Add tasks with different case variations of @WF
    run_command_with_input(&["add", "Task 1 @wf"], "Y\n");
    run_command_with_input(&["add", "Task 2 @WF"], "Y\n");
    run_command_with_input(&["add", "Task 3 @Wf"], "Y\n");
    run_command_with_input(&["add", "Task 4 @work"], "Y\n");

    // List with --hide-waiting should filter out all WF variations
    let output = run_command(&["list", "--hide-waiting"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("Task 1"));
    assert!(!stdout.contains("Task 2"));
    assert!(!stdout.contains("Task 3"));
    assert!(stdout.contains("Task 4"));

    teardown();
}

#[test]
fn test_list_hide_waiting_with_no_results() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Add only waiting tasks
    run_command_with_input(&["add", "Waiting 1 @WF"], "Y\n");
    run_command_with_input(&["add", "Waiting 2 @wf"], "Y\n");

    // List with --hide-waiting should show "No todo items found"
    let output = run_command(&["list", "--hide-waiting"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No todo items found"));

    teardown();
}

#[test]
fn test_list_smart_sorting_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Add tasks with different combinations of due dates and priorities
    run_command_with_input(&["add", "Task A - Due+Pri Due:2026-02-15"], "Y\n");
    run_command(&["pr", "B", "1"]);

    run_command_with_input(&["add", "Task B - Due+Pri Due:2026-02-10"], "Y\n");
    run_command(&["pr", "A", "2"]);

    run_command_with_input(&["add", "Task C - Due only Due:2026-02-05"], "Y\n");

    run_command_with_input(&["add", "Task D - Pri only"], "Y\n");
    run_command(&["pr", "C", "4"]);

    run_command_with_input(&["add", "Task E - Neither"], "Y\n");

    // List and check order
    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find positions
    let task_a_pos = stdout.find("Task A").unwrap();
    let task_b_pos = stdout.find("Task B").unwrap();
    let task_c_pos = stdout.find("Task C").unwrap();
    let task_d_pos = stdout.find("Task D").unwrap();
    let task_e_pos = stdout.find("Task E").unwrap();

    // Expected order:
    // 1. Task B (Due+Pri with priority A, earliest due date in that priority)
    // 2. Task A (Due+Pri with priority B)
    // 3. Task C (Due only)
    // 4. Task D (Pri only)
    // 5. Task E (Neither)

    assert!(
        task_b_pos < task_a_pos,
        "Task B (Due+Pri A) should come before Task A (Due+Pri B)"
    );
    assert!(
        task_a_pos < task_c_pos,
        "Task A (Due+Pri B) should come before Task C (Due only)"
    );
    assert!(
        task_c_pos < task_d_pos,
        "Task C (Due only) should come before Task D (Pri only)"
    );
    assert!(
        task_d_pos < task_e_pos,
        "Task D (Pri only) should come before Task E (Neither)"
    );

    teardown();
}

#[test]
fn test_list_smart_sorting_same_priority_different_due_dates() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Add tasks with same priority but different due dates
    run_command_with_input(&["add", "Task Late Due:2026-03-15"], "Y\n");
    run_command(&["pr", "A", "1"]);

    run_command_with_input(&["add", "Task Early Due:2026-03-10"], "Y\n");
    run_command(&["pr", "A", "2"]);

    run_command_with_input(&["add", "Task Middle Due:2026-03-12"], "Y\n");
    run_command(&["pr", "A", "3"]);

    // List and check order
    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let early_pos = stdout.find("Task Early").unwrap();
    let middle_pos = stdout.find("Task Middle").unwrap();
    let late_pos = stdout.find("Task Late").unwrap();

    // Within same priority (A), should be sorted by earliest due date first
    assert!(
        early_pos < middle_pos,
        "Task Early should come before Task Middle"
    );
    assert!(
        middle_pos < late_pos,
        "Task Middle should come before Task Late"
    );

    teardown();
}

// ===== Project management tests =====

#[test]
fn test_projects_add() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let output = run_command(&["projects", "add", "Backend"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Added project: P:Backend"));

    // Verify projects.json was created
    let content = fs::read_to_string(TEST_PROJECTS_FILE).unwrap();
    assert!(content.contains("Backend"));
    assert!(content.contains("active"));

    teardown();
}

#[test]
fn test_projects_add_duplicate() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command(&["projects", "add", "Backend"]);
    let output = run_command(&["projects", "add", "Backend"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("already exists"));

    teardown();
}

#[test]
fn test_projects_add_case_insensitive_duplicate() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command(&["projects", "add", "Backend"]);
    let output = run_command(&["projects", "add", "backend"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("already exists"));

    teardown();
}

#[test]
fn test_projects_show() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: Some('A'),
            description: "Fix auth".to_string(),
            context: Some("work".to_string()),
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Update docs".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Build UI".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "show", "Backend"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("P:Backend"));
    assert!(stdout.contains("2 open tasks"));
    assert!(stdout.contains("Fix auth"));
    assert!(stdout.contains("Update docs"));
    assert!(!stdout.contains("Build UI"));

    teardown();
}

#[test]
fn test_projects_show_empty() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![make_todo("Buy milk", None, None)]);

    let output = run_command(&["projects", "show", "Backend"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No open tasks for project"));

    teardown();
}

#[test]
fn test_projects_show_excludes_done() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Done task".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/12/01".to_string()),
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Open task".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "show", "Backend"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("1 open task"));
    assert!(stdout.contains("Open task"));
    assert!(!stdout.contains("Done task"));

    teardown();
}

#[test]
fn test_projects_archive() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // First add a project
    run_command(&["projects", "add", "OldProject"]);

    let output = run_command(&["projects", "archive", "OldProject"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Archived project: P:OldProject"));

    // Verify it's archived in the file
    let content = fs::read_to_string(TEST_PROJECTS_FILE).unwrap();
    assert!(content.contains("\"archived\""));

    teardown();
}

#[test]
fn test_projects_archive_not_found() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let output = run_command(&["projects", "archive", "NonExistent"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("not found"));

    teardown();
}

#[test]
fn test_projects_archive_already_archived() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_projects_file(vec![make_project("Done", "archived", None)]);

    let output = run_command(&["projects", "archive", "Done"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("already archived"));

    teardown();
}

#[test]
fn test_projects_list_with_registered() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Register a project with no tasks
    create_test_projects_file(vec![make_project("NewProject", "active", None)]);

    // Create a task with a different project
    let todos = vec![TodoItem {
        priority: None,
        description: "Task 1".to_string(),
        context: None,
        project: Some("Backend".to_string()),
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    }];
    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show both the registered project and the task-derived project
    assert!(stdout.contains("P:Backend"));
    assert!(stdout.contains("P:NewProject"));
    assert!(stdout.contains("Projects:"));

    teardown();
}

#[test]
fn test_projects_list_shows_task_counts() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Task 1".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("2 open tasks"));

    teardown();
}

#[test]
fn test_projects_list_shows_archived_status() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_projects_file(vec![make_project(
        "OldProject",
        "archived",
        Some("2026/01/01"),
    )]);
    create_test_file_with_todos(vec![]);

    let output = run_command(&["projects", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("P:OldProject"));
    assert!(stdout.contains("archived"));
    assert!(stdout.contains("2026/01/01"));

    teardown();
}

#[test]
fn test_projects_review_basic() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: Some('A'),
            description: "Fix auth".to_string(),
            context: Some("work".to_string()),
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Build UI".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    // Review: press 'n' for each project, 'N' for unassigned
    let output = run_command_with_input(&["projects", "review"], "n\nn\nN\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Project Review"));
    assert!(stdout.contains("P:Backend"));
    assert!(stdout.contains("P:Frontend"));
    assert!(stdout.contains("Fix auth"));
    assert!(stdout.contains("Build UI"));
    assert!(stdout.contains("Review complete!"));
    assert!(stdout.contains("Reviewed 2 projects"));

    // Verify projects.json was created with last_reviewed dates
    let projects_content = fs::read_to_string(TEST_PROJECTS_FILE).unwrap();
    assert!(projects_content.contains("Backend"));
    assert!(projects_content.contains("Frontend"));
    assert!(projects_content.contains("last_reviewed"));

    teardown();
}

#[test]
fn test_projects_review_quit_early() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Task A".to_string(),
            context: None,
            project: Some("Alpha".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task B".to_string(),
            context: None,
            project: Some("Beta".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    // Quit after first project
    let output = run_command_with_input(&["projects", "review"], "q\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("P:Alpha"));
    assert!(stdout.contains("Reviewed 1 projects"));

    teardown();
}

#[test]
fn test_projects_review_add_task() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![TodoItem {
        priority: None,
        description: "Existing task".to_string(),
        context: None,
        project: Some("Backend".to_string()),
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    }];
    create_test_file_with_todos(todos);

    // Add a task during review, then next, decline unassigned
    let output =
        run_command_with_input(&["projects", "review"], "a\nNew API endpoint @work\nn\nN\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Added: \"New API endpoint\" to P:Backend"));
    assert!(stdout.contains("Review complete!"));

    // Verify the task was saved
    let content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(content.contains("New API endpoint"));
    assert!(content.contains("Backend"));

    teardown();
}

#[test]
fn test_projects_review_unassigned_tasks() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Assigned task".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Orphan task".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    // Review Backend, then assign orphan task to Frontend
    let output = run_command_with_input(&["projects", "review"], "n\nY\n2 Frontend\nd\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Unassigned Tasks"));
    assert!(stdout.contains("Orphan task"));
    assert!(stdout.contains("Assigned \"Orphan task\" to P:Frontend"));

    // Verify the assignment was saved
    let content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    let saved_todos: Vec<TodoItem> = serde_json::from_str(&content).unwrap();
    assert_eq!(saved_todos[1].project, Some("Frontend".to_string()));

    teardown();
}

#[test]
fn test_projects_review_no_projects_no_tasks() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    create_test_file_with_todos(vec![]);

    let output = run_command_with_input(&["projects", "review"], "");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No active projects to review and no unassigned tasks"));

    teardown();
}

#[test]
fn test_projects_review_skips_archived() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    // Create an archived project and an active one
    create_test_projects_file(vec![
        make_project("Archived", "archived", None),
        make_project("Active", "active", None),
    ]);

    let todos = vec![
        TodoItem {
            priority: None,
            description: "Old task".to_string(),
            context: None,
            project: Some("Archived".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
        TodoItem {
            priority: None,
            description: "Current task".to_string(),
            context: None,
            project: Some("Active".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
            due_date: None,
        },
    ];
    create_test_file_with_todos(todos);

    let output = run_command_with_input(&["projects", "review"], "n\nN\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should review Active but not Archived
    assert!(stdout.contains("P:Active"));
    assert!(stdout.contains("Current task"));
    assert!(stdout.contains("Reviewed 1 projects"));
    // Archived project should not appear in review flow
    assert!(!stdout.contains("── P:Archived"));

    teardown();
}

#[test]
fn test_projects_show_case_insensitive() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let todos = vec![TodoItem {
        priority: None,
        description: "Task 1".to_string(),
        context: None,
        project: Some("Backend".to_string()),
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: None,
        due_date: None,
    }];
    create_test_file_with_todos(todos);

    let output = run_command(&["projects", "show", "backend"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Task 1"));

    teardown();
}
