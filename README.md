# todo-cli

[![CI](https://github.com/rajshahuk/todo-cli/workflows/CI/badge.svg)](https://github.com/rajshahuk/todo-cli/actions)

A fast, colorful command-line todo list manager written in Rust. Keep track of your tasks with priorities, contexts, projects, and tags—all stored in a simple JSON file.

## Features

- 🎨 **Color-coded output** for easy scanning
- 📊 **Priority management** (A-Z, where A is highest)
- 🏷️ **Organize with contexts** (`@work`, `@home`), **projects** (`P:ProjectName`), and **tags** (`T:urgent`)
- ✅ **Track completion** with automatic date tracking
- 📅 **Due dates** - set absolute or relative due dates, with automatic sorting and overdue highlighting
- 🎯 **Smart sorting** - items with both due date and priority automatically appear first
- 🔍 **Flexible listing** - view all tasks or filter by completion status
- ⏰ **Age-based filtering** - find todos older than a specific duration (e.g., `+1d`, `+2w`, `+1m`)
- 🚫 **Hide waiting items** - filter out tasks marked as @WF (waiting for)
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

Add a task with a due date:
```bash
todo-cli add "Submit report Due:2026-06-30"      # Absolute date
todo-cli add "Follow up on email Due:+3d"        # Relative date (3 days from now)
todo-cli add "Call client Due:+1w @work"         # Combine with other metadata
```

The metadata markers (`@`, `P:`, `T:`, `Due:`) can appear anywhere in your description:
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

Hide waiting items (marked with @WF):
```bash
todo-cli list --hide-waiting    # Filter out items with @WF context
```

Combine filters and flags:
```bash
todo-cli list --all --pr           # All todos sorted by priority
todo-cli list --all +1w            # All todos (including done) older than 1 week
todo-cli list --pr +3d             # Uncompleted todos older than 3 days, sorted by priority
todo-cli list --hide-waiting --pr  # Active tasks (no @WF) sorted by priority
```

Example output:
```
2 (A) Due:2026/01/15 S:2025/11/30 Send email @work T:important
3 (B) Due:2026/01/20 S:2025/11/30 Review code P:ProjectX T:review T:backend
4 Due:2026/01/25 S:2025/11/30 Follow up with client @work
5 (C) S:2025/11/30 Plan meeting P:ProjectX
1 S:2025/11/30 Buy milk @shopping P:Personal T:urgent
```

Note: Items are automatically sorted by importance:
1. **Items with BOTH due date AND priority** appear first (sorted by priority, then by due date)
2. **Items with due date only** appear next (sorted by earliest due date)
3. **Items with priority only** appear next (sorted by priority)
4. **Items with neither** appear last (sorted by line number)

Overdue items are highlighted in red and bold.

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

### Editing Tasks

Edit any field of an existing todo item:
```bash
todo-cli edit 1
```

You'll be prompted interactively for each field:
```
Editing todo item 1:
Press Enter to keep current value, or type new value

Description [Buy milk]: Buy groceries
Priority (A-Z, or 'clear') [none]: A
Context (without @) [none]: shopping
Project (without P:) [none]: Personal
Tags (comma-separated, without T:) [none]: urgent, today
Due date (YYYY-MM-DD, +3d, +2w, or 'clear') [none]: 2026-06-30

Todo item 1 updated successfully
```

Tips for editing:
- **Keep current value**: Just press Enter without typing anything
- **Clear a field**: Type `clear` or `none` to remove the value
- **Multiple tags**: Separate with commas (e.g., `urgent, important, today`)
- **Priority**: Single letter A-Z, or `clear` to remove
- **Due dates**: Use absolute (YYYY-MM-DD) or relative (+3d, +2w, +1m, +1y) formats

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
| `add "description"` | Add a new todo item (supports `@context`, `P:project`, `T:tag`, `Due:date`) |
| `list` | Show uncompleted items (smart sorted: items with due date+priority first) |
| `list --all` | Show all items including completed |
| `list --pr` | Show items sorted by priority (preserves smart sorting for items with due dates) |
| `list --hide-waiting` | Hide items marked as waiting (@WF) |
| `list +<time>` | Filter by age (e.g., `+1d`, `+2w`, `+3m`, `+1y`) |
| `list --all +<time>` | Show all items older than specified duration |
| `list --pr +<time>` | Show old items sorted by priority |
| `list --hide-waiting --pr` | Active items (no @WF) sorted by priority |
| `edit <number>` | Edit any field including due date interactively |
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

## Due Dates

Set deadlines for your tasks using the `Due:` marker. Tasks with due dates are automatically sorted to the top of your list, with the earliest dates first. Overdue items are highlighted in red for visibility.

### Absolute Dates

Specify an exact due date using YYYY-MM-DD or YYYY/MM/DD format:

```bash
todo-cli add "Submit tax return Due:2026-04-15"
todo-cli add "Renew passport Due:2026/08/20"
```

### Relative Dates

Use relative dates to set deadlines based on the current date:

```bash
todo-cli add "Follow up on proposal Due:+3d"    # 3 days from now
todo-cli add "Weekly review Due:+1w"            # 1 week from now
todo-cli add "Quarterly planning Due:+1m"       # 1 month from now
todo-cli add "Annual review Due:+1y"            # 1 year from now
```

Supported units:
- `d` - days
- `w` - weeks (7 days)
- `m` - months (30 days)
- `y` - years (365 days)

### Managing Due Dates

**Edit a due date:**
```bash
todo-cli edit 1
# At the "Due date" prompt, enter a new date or press Enter to keep current
```

**Clear a due date:**
```bash
todo-cli edit 1
# At the "Due date" prompt, type "clear" or "none"
```

### Smart Automatic Sorting

When you list your todos, items are automatically sorted by importance to help you focus on what matters most:

1. **Items with BOTH due date AND priority** - These are your most important tasks
   - Sorted by priority first (A before B before C, etc.)
   - Within the same priority, sorted by earliest due date first

2. **Items with due date only** - Sorted by earliest due date first

3. **Items with priority only** - Sorted by priority (A before B before C, etc.)

4. **Items with neither** - Sorted by line number (creation order)

**Overdue items** are highlighted in **red** and **bold** for immediate visibility.

This smart sorting ensures that urgent, important tasks with deadlines always appear at the top of your list, making it easy to focus on what needs your attention first.

### Examples

```bash
# Add a task with a tight deadline
todo-cli add "Complete project proposal Due:+2d @work P:ClientA T:urgent"

# View your upcoming deadlines
todo-cli list

# See only prioritized tasks with due dates
todo-cli list --pr
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
- **Due dates**: Normal text (red bold for overdue items)
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
    "done_date": null,
    "due_date": "2026/01/15"
  },
  {
    "priority": null,
    "description": "Send email",
    "context": "work",
    "project": null,
    "tags": ["important", "today"],
    "start_date": "2025/11/30",
    "done_date": "2025/12/01",
    "due_date": null
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
| `due_date` | string or null | Date due (yyyy/mm/dd), from `Due:` marker |

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
- **38 unit tests** - Testing metadata parsing, JSON serialization, age filtering, and date handling
- **52 integration tests** - Testing all CLI commands end-to-end including edit, due dates, filtering, and smart sorting

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
todo-cli list --pr --hide-waiting  # Review active prioritized tasks
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

**Focus on actionable tasks:**
```bash
todo-cli list --hide-waiting  # Hide items waiting on others (@WF)
```

**Upcoming deadlines:**
```bash
todo-cli list  # Items with due dates automatically appear first
```

**Set deadlines for follow-ups:**
```bash
todo-cli add "Follow up with client Due:+3d @work"  # Due in 3 days
```

**Create high-priority tasks with deadlines:**
```bash
todo-cli add "Complete proposal Due:+2d @work P:ClientA"
todo-cli pr A 1  # Set priority A - this will appear at the very top
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
