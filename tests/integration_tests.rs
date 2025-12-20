use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::sync::Mutex;

const TEST_TODO_FILE: &str = "todo.json";

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
}

fn setup() {
    // Remove test file if it exists
    let _ = fs::remove_file(TEST_TODO_FILE);
}

fn teardown() {
    // Clean up test file
    let _ = fs::remove_file(TEST_TODO_FILE);
}

fn get_binary_path() -> String {
    // Check if release binary exists, otherwise use debug
    if std::path::Path::new("./target/release/todo-cli").exists() {
        "./target/release/todo-cli".to_string()
    } else {
        "./target/debug/todo-cli".to_string()
    }
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

fn make_todo(description: &str, priority: Option<char>, done_date: Option<&str>) -> TodoItem {
    TodoItem {
        priority,
        description: description.to_string(),
        context: None,
        project: None,
        tags: vec![],
        start_date: "2025/11/29".to_string(),
        done_date: done_date.map(|s| s.to_string()),
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
        },
        TodoItem {
            priority: None,
            description: "Send email".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/11/30".to_string()),
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
        },
        TodoItem {
            priority: None,
            description: "Send email".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: Some("2025/11/30".to_string()),
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
        },
        TodoItem {
            priority: Some('A'),
            description: "Task A".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
        },
        TodoItem {
            priority: Some('B'),
            description: "Task B".to_string(),
            context: None,
            project: None,
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
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

    let output = run_command(&["projects"]);
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
    };

    create_test_file_with_todos(vec![todo]);

    let output = run_command(&["projects"]);
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
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 3".to_string(),
            context: None,
            project: Some("API".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
        },
    ];

    create_test_file_with_todos(todos);

    let output = run_command(&["projects"]);
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
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
        },
        TodoItem {
            priority: None,
            description: "Task 3".to_string(),
            context: None,
            project: Some("Backend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
        },
    ];

    create_test_file_with_todos(todos);

    let output = run_command(&["projects"]);
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
        },
        TodoItem {
            priority: None,
            description: "Task 2".to_string(),
            context: None,
            project: Some("Frontend".to_string()),
            tags: vec![],
            start_date: "2025/11/29".to_string(),
            done_date: None,
        },
    ];

    create_test_file_with_todos(todos);

    let output = run_command(&["projects"]);
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
