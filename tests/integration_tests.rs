use std::fs;
use std::process::Command;
use std::sync::Mutex;

const TEST_TODO_FILE: &str = "todo.txt";

// Global lock to ensure tests run serially
static TEST_LOCK: Mutex<()> = Mutex::new(());

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
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    child.wait_with_output().expect("Failed to wait for command")
}

fn create_test_file_with_content(content: &str) {
    fs::write(TEST_TODO_FILE, content).expect("Failed to write test file");
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
    assert!(content.contains("S:"));

    teardown();
}

#[test]
fn test_add_todo_with_metadata() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    run_command_with_input(&["add", "Buy milk @shopping P:Personal T:urgent"], "Y\n");

    let content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(content.contains("Buy milk @shopping P:Personal T:urgent"));
    assert!(content.contains("S:"));

    teardown();
}

#[test]
fn test_list_empty() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();
    create_test_file_with_content("");

    let output = run_command(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No todo items found") || stdout.is_empty());

    teardown();
}

#[test]
fn test_list_filters_done_items() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29\nSend email S:2025/11/29 D:2025/11/30\n";
    create_test_file_with_content(content);

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

    let content = "Buy milk S:2025/11/29\nSend email S:2025/11/29 D:2025/11/30\n";
    create_test_file_with_content(content);

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

    let content = "(C) Task C S:2025/11/29\n(A) Task A S:2025/11/29\n(B) Task B S:2025/11/29\n";
    create_test_file_with_content(content);

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

    let content = "Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    let output = run_command(&["pr", "a", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Set priority"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("(A) Buy milk"));

    teardown();
}

#[test]
fn test_change_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "(A) Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    run_command(&["pr", "b", "1"]);

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("(B) Buy milk"));
    assert!(!updated_content.contains("(A) Buy milk"));

    teardown();
}

#[test]
fn test_clear_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "(A) Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    let output = run_command(&["pr", "clear", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Cleared priority"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(!updated_content.contains("(A)"));
    assert!(updated_content.contains("Buy milk"));

    teardown();
}

#[test]
fn test_mark_done() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    let output = run_command_with_input(&["done", "1"], "Y\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("marked as done"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("D:"));

    teardown();
}

#[test]
fn test_mark_done_cancelled() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    let output = run_command_with_input(&["done", "1"], "N\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Cancelled"));

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(!updated_content.contains("D:"));

    teardown();
}

#[test]
fn test_mark_done_already_done() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29 D:2025/11/30\n";
    create_test_file_with_content(content);

    let output = run_command_with_input(&["done", "1"], "Y\n");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("already marked as done"));

    teardown();
}

#[test]
fn test_mark_done_invalid_number() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    let output = run_command(&["done", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("does not exist"));

    teardown();
}

#[test]
fn test_priority_invalid_number() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    let output = run_command(&["pr", "a", "99"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("does not exist"));

    teardown();
}

#[test]
fn test_lowercase_priority_converted() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Buy milk S:2025/11/29\n";
    create_test_file_with_content(content);

    run_command(&["pr", "c", "1"]);

    let updated_content = fs::read_to_string(TEST_TODO_FILE).unwrap();
    assert!(updated_content.contains("(C)"));

    teardown();
}

#[test]
fn test_list_shows_line_numbers() {
    let _lock = TEST_LOCK.lock().unwrap();
    setup();

    let content = "Task 1 S:2025/11/29\nTask 2 S:2025/11/29\nTask 3 S:2025/11/29\n";
    create_test_file_with_content(content);

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

    let content = "(A) Buy milk S:2025/11/29 D:2025/11/30\n";
    create_test_file_with_content(content);

    let output = run_command(&["list", "--all"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("(A)"));
    assert!(stdout.contains("Buy milk"));

    teardown();
}
