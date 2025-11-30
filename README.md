# todo-cli

A fast, colorful command-line todo list manager written in Rust. Keep track of your tasks with priorities, contexts, projects, and tags—all stored in a simple JSON file.

## Features

- 🎨 **Color-coded output** for easy scanning
- 📊 **Priority management** (A-Z, where A is highest)
- 🏷️ **Organize with contexts** (`@work`, `@home`), **projects** (`P:ProjectName`), and **tags** (`T:urgent`)
- ✅ **Track completion** with automatic date tracking
- 🔍 **Flexible listing** - view all tasks or filter by completion status
- 📁 **JSON storage** - human-readable, easy to backup or sync

## Quick Start

### Build the application

```bash
cargo build --release
```

### Add your first todo

```bash
./target/release/todo-cli add "Buy groceries @home"
```

On first run, you'll be prompted to create the `todo.json` file in your current directory.

### View your todos

```bash
./target/release/todo-cli list
```

That's it! You're managing todos from the command line.

## Usage Examples

### Adding Tasks

Add a simple task:
```bash
todo-cli add "Call the dentist"
```

Add a task with context, project, and tags:
```bash
todo-cli add "Review pull request @work P:Backend T:urgent T:review"
```

The metadata markers (`@`, `P:`, `T:`) can appear anywhere in your description:
```bash
todo-cli add "Email team about P:Launch campaign tomorrow"
# Result: description="Email team about campaign tomorrow", project="Launch"
```

### Viewing Tasks

List uncompleted tasks:
```bash
todo-cli list
```

View all tasks (including completed):
```bash
todo-cli list --all
```

Sort by priority:
```bash
todo-cli list --pr
```

Combine flags:
```bash
todo-cli list --all --pr
```

Example output:
```
2 (A) S:2025/11/30 Send email @work T:important
3 (B) S:2025/11/30 Review code P:ProjectX T:review T:backend
1 S:2025/11/30 Buy milk @shopping P:Personal T:urgent
```

### Setting Priorities

Set a priority (A is highest, Z is lowest):
```bash
todo-cli pr a 2    # Set priority A on item 2
todo-cli pr B 5    # Set priority B (accepts lowercase)
```

Remove a priority:
```bash
todo-cli pr clear 2
```

### Completing Tasks

Mark a task as done:
```bash
todo-cli done 1
```

You'll be asked to confirm:
```
Mark this item as done?
  Buy milk @shopping P:Personal S:2025/11/30
(Y/N):
```

### Viewing Projects

List all unique projects across all todos:
```bash
todo-cli projects
```

Example output:
```
Projects:
  P:Backend
  P:Frontend
  P:Website
```

This command shows all projects in alphabetical order, including those from completed items.

## Commands Reference

| Command | Description |
|---------|-------------|
| `add "description"` | Add a new todo item |
| `list` | Show uncompleted items |
| `list --all` | Show all items including completed |
| `list --pr` | Show items sorted by priority |
| `done <number>` | Mark item as done (with confirmation) |
| `pr <priority> <number>` | Set priority A-Z on an item |
| `pr clear <number>` | Remove priority from an item |
| `projects` | List all unique projects |

## Organizing Your Todos

### Contexts (one per task)

Use `@` to specify where or when a task should be done:
- `@work` - Tasks to do at work
- `@home` - Home tasks
- `@computer` - Tasks requiring a computer
- `@errands` - Things to do while out

```bash
todo-cli add "Send meeting notes @work"
```

If you specify multiple contexts, only the first is saved:
```bash
todo-cli add "Task @first @second"  # Only @first is saved
```

### Projects (one per task)

Use `P:` or `p:` (case-insensitive) to group tasks by project:
- `P:Website` - Website redesign tasks
- `p:Personal` - Personal project tasks
- `P:Learning` - Learning and study tasks

```bash
todo-cli add "Update landing page P:Website"
todo-cli add "Call dentist p:personal"  # Lowercase works too
```

### Tags (multiple per task)

Use `T:` or `t:` (case-insensitive) to add flexible labels. You can add as many as needed:
- `T:urgent` - High priority tasks
- `t:waiting` - Waiting on someone else
- `T:quick` - Quick tasks (< 5 minutes)

```bash
todo-cli add "Fix login bug T:urgent T:bug t:frontend"  # Mixed case works
```

## Color Scheme

When viewing your list, different elements are color-coded for quick identification:

- **Line numbers**: Cyan
- **Priorities**: Magenta
- **Contexts** (`@`): Green
- **Projects** (`P:`): Yellow
- **Tags** (`T:`): Bright blue

Colors are optimized for dark terminal backgrounds.

## Data Format

Todos are stored in `todo.json` in your current working directory. The file is a JSON array of todo objects:

```json
[
  {
    "priority": "A",
    "description": "Buy milk",
    "context": "shopping",
    "project": "Personal",
    "tags": ["urgent"],
    "start_date": "2025/11/30",
    "done_date": null
  },
  {
    "priority": null,
    "description": "Send email",
    "context": "work",
    "project": null,
    "tags": ["important", "today"],
    "start_date": "2025/11/30",
    "done_date": "2025/12/01"
  }
]
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `priority` | string or null | Single character A-Z (A = highest priority) |
| `description` | string | The task description (metadata markers removed) |
| `context` | string or null | Single context from `@context` marker |
| `project` | string or null | Single project from `P:project` marker |
| `tags` | array | Zero or more tags from `T:tag` markers |
| `start_date` | string | Date created (yyyy/mm/dd), auto-generated |
| `done_date` | string or null | Date completed (yyyy/mm/dd), set when done |

The JSON format makes it easy to:
- Back up your todos (just copy the file)
- Sync across devices (use Dropbox, Git, etc.)
- Write scripts to analyze your tasks
- Manually edit if needed

## Development

### Prerequisites

- Rust 1.70+ (uses 2021 edition)
- Cargo (comes with Rust)

### Dependencies

- **clap** (4.5) - Command-line argument parsing
- **colored** (2.1) - Terminal colors
- **chrono** (0.4) - Date handling
- **serde** (1.0) - Serialization
- **serde_json** (1.0) - JSON support

### Building

Debug build (faster compilation):
```bash
cargo build
./target/debug/todo-cli
```

Release build (optimized):
```bash
cargo build --release
./target/release/todo-cli
```

### Testing

Run all tests:
```bash
cargo test
```

Run only unit tests:
```bash
cargo test --bin todo-cli
```

Run only integration tests:
```bash
cargo test --test integration_tests
```

The test suite includes:
- **14 unit tests** - Testing metadata parsing and JSON serialization
- **22 integration tests** - Testing all CLI commands end-to-end

## Tips

### Getting Started

1. Create a `todo.json` file in a central location (e.g., `~/todos/`)
2. Add an alias to your shell config:
   ```bash
   alias t='cd ~/todos && todo-cli'
   ```
3. Now you can quickly manage todos from anywhere: `t add "Task"`, `t list`

### Workflow Suggestions

**Morning routine:**
```bash
todo-cli list --pr  # Review prioritized tasks
```

**Capture tasks quickly:**
```bash
todo-cli add "Task" T:inbox  # Tag for later processing
```

**Weekly review:**
```bash
todo-cli list --all  # Review completed and pending tasks
```

**Project focus:**
```bash
todo-cli projects                      # View all projects
todo-cli list --pr | grep "P:Website"  # See all website tasks
```

## License

This project is open source. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure all tests pass before submitting a pull request.

```bash
cargo test
cargo build --release
```
