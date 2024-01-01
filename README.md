# DoTodo

Do your todos!
A todo list to organise your needs :)

## Pictures
|                          <img width="1414" src="https://github.com/SleepySwords/do_todo/assets/33922797/b572a1af-3d70-46d5-ac24-17887532fbae"> Tags                          | <img width="1403" alt="Screenshot 2023-12-14 at 11 03 48 pm" src="https://github.com/SleepySwords/do_todo/assets/33922797/26429f86-15ee-492a-9c37-af187687c47f"> Fuzzy finder |
|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------:|:-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------:|
| <img width="1405" alt="Screenshot 2024-01-01 at 3 56 09 pm" src="https://github.com/SleepySwords/do_todo/assets/33922797/6bfd0088-8c31-4fa7-bd47-b0e3c96f045f"> **Subtasks** |

Be sure to backup your data!

## Install from source
1. Run `git clone git@github.com/SleepySwords/do_todo.git`
2. Run `cd do_todo`
3. Run `cargo install --path ./`

## Configuration/data paths
Paths to your data.

For the following, replace `YOUR_USERNAME` with your username.

| Files  | Windows                                                  | Linux                                               | MacOS                                                                |
|--------|----------------------------------------------------------|-----------------------------------------------------|----------------------------------------------------------------------|
| Config | `C:\Users\YOUR_USERNAME\AppData\Local\dotodo\config.yml` | `/home/YOUR_USERNAME/.config/dotodo/config.yml`     | `/Users/YOUR_USERNAME/Library/Application Support/dotodo/config.yml` |
| Tasks  | `C:\Users\YOUR_USERNAME\AppData\Local\dotodo\data.json`  | `/home/YOUR_USERNAME/.local/share/dotodo/data.json` | `/Users/YOUR_USERNAME/Library/Application Support/dotodo/data.json`  |

## Keybindings

### Main default binds

| Key          | Action                      |
|--------------|-----------------------------|
| `1`          | Select tasklist             |
| `2`          | Select completed tasklist   |
| `k`          | Move up                     |
| `j`          | Move down                   |
| `Ctrl` + `n` | Move up in the fuzzy list   |
| `Ctrl` + `p` | Move down in the fuzzy list |
| `x`          | Open the help menu          |
| `q`          | Quit do_todo                |

### Task list default binds
| Key    | Action
|--------|--------------------------------------------------------|
| `a`    | Adds a task|
| `c`    | Completes the selected task |            | `d`    | Delete the selected task|
| `e`    | Edits the selected task|
| `t`    | Add or remove the tags from this task or project|
| `k`    | Gives selected task lower| priority
| `J`    | Moves the task down on the task list|
| `K`    | Moves the task up on the task list|
| `s`    | Sorts tasks (by priority)|
| `S`    | Toggles automatic task sort|
| `enter` | Open/closes the subtask|                  
| `L`     | Make the selected task a subtask of above             |
| `H`.    | Make the selected task not a subtask of the parent |