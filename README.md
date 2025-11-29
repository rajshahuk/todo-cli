# todo-cli

This is a command line application written in rust. It uses the clap framework to parse command
line arguments.

Here are the basics of how this application works:

   * Todo items are managed a text file in this directory. The file is called `todo.txt` and the format of the file is very similar to that which is used in todo.gtd 
   * Each todo is on a single line in the file
   * When an items is added to the file we always add the start date of the todo which is the current date. The start date is denoted by S:[DATE]
   * All dates are the format yyyy/mm/dd where yyyy = year, mm = month, dd = day
   * An item is added to the todo list by passing the `add` command to the programme
   * Outstanding items can be viewed by the user by passing in the `list` command to the programme. Outstanding items are ones that are not yet done. The `list` command prints the line number of the todo item in the file
   * An item can be marked as done by passing the `done` command with the todo item number from the list command. When this is run the todo item in the file is marked with D:[DATE]
   * A todo item can optionally have a context. This is denoted by the `@` symbol. Example contexts might be `@email` `@teams` etc.
   * A todo item can optionally have a project associataed with it. This is denoted by `P:[PROJECT_NAME]`, Examples could be, `P:ProjectOne`, `P:ProjectTwo`
   * A todo item can optionally have multiple tags associataed with it. This is denoted by `T:[TAG_NAME]`, Examples could be, `T:TagOne`, `T:TagTwo`
   * When viewing the `list` of todo items, line numbers, contexts and tags should be displayed in a different colour.
   * A todo item can be prioritised by passing in the `pr` command followed by a single character. For example, `todo-cli pr a 12` would be prioritise todo item 12 with priority A. All priority values should be put in uppercase and should always appear at the start of the line in the todo file.
   
## Clarification of Requirements

### Adding an item to the todo file

The syntax for adding a todo item is as follows:

```
todo-cli add "Buy milk @shopping P:Personal"
```

### Line format

```
(A) Buy milk @shopping P:Personal T:urgent S:2025/11/29 D:2025/11/30 T:Tescos
```

   * Priority always comes at the beginning of the line
   * start date, done date, projects, contexts and tags could appear anywhere on the line

### Done bevahiour

   * D:[DATE] can be added anywhere on the line, including the end of the line
   * The piority marker can be kept when an item is marked as done
   * Ask the user for confirmation when they mark an items as done. Display the original todo item in the confirmation message. A simple Y/N will suffice
   * If the todo item does not exist or it is already marked as done  then display a suitable error


### Priority command

   * The valid range is A-Z
   * You can change the priority of an exising item
   * There should be a way to clear the priority of an item i.e. `todo pr clear 12`
   * You can except either lowercase or uppercase for the priority
   * A is higher priority than B
   * The lowest priority is Z
   * If the todo item does not exist then display a suitable error

### List Output

   * By default the list output should exclude items that are done.
   * If you run the list comamand with `list --all` then all the items should be shown
   * The output should show the actual line number in the file
   * If you run the commend with `list --pr` then items should be ordered by priority and the remaining items should be ordered by the file order.
   * done items should respect priority sorting too if --all is used

### Filtering

At this point in time there is no need to be able to filter by project, tag, context etc

### File Location

   * The file should be in the current working directory
   * If the file doesn't exist prompt the user if they want to create it with a clear message on where it will be created.

### Colour Scheme

   * Pick accessible colours in the terminal when running no a dark background.
   * line numbers, contexts, projects and tags should all have different colours
   * priority should have its own colour as well

### Multiple values

   * There can only be one project per item - it is optional
      * if someone writes P:Project1 P:Project2 then take the first one.
   * There can only be one context per item - it is optional
   * There can be multiple tags per item - they are also optional. Each tag will be prefixed with T:

### Metadata

   * if someone writes "Email about P:ProjectX", this can be treated as part of the whole todo item and the project should be parsed as well.
