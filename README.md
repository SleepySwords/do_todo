# DoTodo

Do your todos!
A todo list to organise your needs :)

<img width="1414" src="https://github.com/SleepySwords/do_todo/assets/33922797/b572a1af-3d70-46d5-ac24-17887532fbae">
<img width="1403" alt="Screenshot 2023-12-14 at 11 03 48 pm" src="https://github.com/SleepySwords/do_todo/assets/33922797/26429f86-15ee-492a-9c37-af187687c47f">

This project in its current state is extremely unstable. Features may break existing data or configs. Use at your own risk and make sure to backup your data!

## Install from source
1. Run `git clone git@github.com/SleepySwords/do_todo.git`
2. Run `cd do_todo`
3. Run `cargo install --path ./`

## Configuration/data paths
Paths to your data.

For the following, replace `YOUR_USERNAME` with your username.

### Windows
Theme: `C:\Users\YOUR_USERNAME\AppData\Local\dotodo`

Tasks: `C:\Users\YOUR_USERNAME\AppData\Local\dotodo`

### Linux
Theme: `/home/YOUR_USERNAME/.config/dotodo`

Tasks: `/home/YOUR_USERNAME/.local/share/dotodo`

### MasOS
Theme: `/Users/YOUR_USERNAME/Library/Application Support/dotodo`

Tasks: `/Users/YOUR_USERNAME/Library/Application Support/dotodo`

## Keybindings

| Key          | Action                      |
|--------------|-----------------------------|
| `1`          | Select tasklist             |
| `2`          | Select completed tasklist   |
| `h`          | Move up                     |
| `j`          | Move down                   |
| `Ctrl` + `n` | Move up in the fuzzy list   |
| `Ctrl` + `p` | Move down in the fuzzy list |
| `x`          | Open the help menu          |
| `q`          | Quit do_todo                |
