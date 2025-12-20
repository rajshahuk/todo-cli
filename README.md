# todo-cli

[![CI](https://github.com/rajshahuk/todo-cli/workflows/CI/badge.svg)](https://github.com/rajshahuk/todo-cli/actions)

A fast, colorful command-line todo list manager written in Rust. Keep track of your tasks with priorities, contexts, projects, and tags—all stored in a simple JSON file.

## Features

- 🎨 **Color-coded output** for easy scanning
- 📊 **Priority management** (A-Z, where A is highest)
- 🏷️ **Organize with contexts** (`@work`, `@home`), **projects** (`P:ProjectName`), and **tags** (`T:urgent`)
- ✅ **Track completion** with automatic date tracking
- 🔍 **Flexible listing** - view all tasks or filter by completion status
- ⏰ **Age-based filtering** - find todos older than a specific duration (e.g., `+1d`, `+2w`, `+1m`)
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

Filter by age (show only old tasks):
```bash
todo-cli list +1d   # Todos older than 1 day
todo-cli list +2w   # Todos older than 2 weeks
todo-cli list +3m   # Todos older than 3 months
todo-cli list +1y   # Todos older than 1 year
```

Supported time units for age filtering:
- `d` = days
- `w` = weeks
- `m` = months
- `y` = years

Combine filters and flags:
```bash
todo-cli list --all --pr        # All todos sorted by priority
todo-cli list --all +1w         # All todos (including done) older than 1 week
todo-cli list --pr +3d          # Uncompleted todos older than 3 days, sorted by priority
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
| `list +<time>` | Filter by age (e.g., `+1d`, `+2w`, `+3m`, `+1y`) |
| `list --all +<time>` | Show all items older than specified duration |
| `list --pr +<time>` | Show old items sorted by priority |
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

## Age-Based Filtering

Find tasks that have been sitting around for too long using the age filter. This helps you identify stale todos that might need attention or cleanup.

### How It Works

The age filter compares each todo's `start_date` (when it was created) with the current date. Only todos **older than** the specified duration are shown.

### Usage Format

Use the format `+<number><unit>` where:
- Number must be positive (e.g., `1`, `7`, `30`)
- Unit can be:
  - `d` - days
  - `w` - weeks (7 days)
  - `m` - months (30 days)
  - `y` - years (365 days)

### Examples

Find todos that have been pending for a while:
```bash
todo-cli list +1d    # Older than 1 day
todo-cli list +1w    # Older than 1 week (7 days)
todo-cli list +2w    # Older than 2 weeks (14 days)
todo-cli list +1m    # Older than 1 month (30 days)
todo-cli list +3m    # Older than 3 months (90 days)
todo-cli list +1y    # Older than 1 year (365 days)
```

### Common Use Cases

**Weekly review** - Find tasks older than a week:
```bash
todo-cli list +1w
```

**Stale task cleanup** - Find tasks that have been pending for months:
```bash
todo-cli list +3m
```

**Include completed tasks** - Review old completed tasks:
```bash
todo-cli list --all +1m
```

**Priority focus** - Old high-priority tasks that need attention:
```bash
todo-cli list --pr +2w
```

### Notes

- The filter uses the `start_date` field (when the todo was created)
- Months are approximated as 30 days, years as 365 days
- The filter works with both `--all` and `--pr` flags
- Without `--all`, only uncompleted todos are considered
- Invalid formats show a helpful error message

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

- Rust 1.70+ (uses 2024 edition)
- Cargo (comes with Rust)

### Continuous Integration

This project uses GitHub Actions for continuous integration. The CI pipeline:

- **Tests** on multiple platforms (Ubuntu, macOS, Windows)
- **Lints** code with rustfmt and clippy
- **Measures code coverage** with tarpaulin

The workflow runs automatically on:
- Push to `main` branch
- Pull requests to `main` branch

To run the same checks locally before pushing:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release
cargo build --release
```

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
- **25 unit tests** - Testing metadata parsing, JSON serialization, and age filtering
- **34 integration tests** - Testing all CLI commands end-to-end

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
todo-cli list +1w    # Check tasks older than a week
```

**Project focus:**
```bash
todo-cli projects                      # View all projects
todo-cli list --pr | grep "P:Website"  # See all website tasks
```

**Stale task cleanup:**
```bash
todo-cli list +1m    # Find tasks sitting for over a month
```

## License

This project is open source. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure all checks pass before submitting a pull request:

```bash
# Format code
cargo fmt

# Run linter (no warnings allowed)
cargo clippy -- -D warnings

# Run all tests
cargo test

# Build release
cargo build --release
```

The CI pipeline will automatically run these checks on your pull request.
