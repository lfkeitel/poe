# poe

Poe is a plain ol' editor. It's line-based text editor inspired by ed.

## Building

1. Clone the repo
2. Run `cargo build`

## Usage

`poe [FILENAME]`

Poe starts at a command mode prompt "0 >". The number is the current line number.
The right arrow indicates command mode.

### Modes

Poe has three modes:

- `>` - Command mode. In this mode, commands can be used to manipulate the file.
  Edit and insert mode can be entered from here.
- `#` - Edit mode. Edit the current line of text.
- `+` - Insert mode. Insert a new line.

### Commands

- `NUM` - Set current line number. Lines start at 0.
- `?` - Print help text.
- `c [NUM]` - Print context lines around current line, defaults to 2 lines.
- `d` - Delete current line.
- `e` - Edit current line.
- `f [TEXT]` - Find text below current line.
- `F [TEXT]` - Find text above current line.
- `i` - Insert new line below current line.
- `I` - Insert new line above current line.
- `m` - Print editor data.
- `q` - Quit editor.
- `p [NUM]` - Print current line. If given a number, will set the current line
  and print it.
- `w [FILENAME]` - Write file. If FILENAME is given the file will be written
  there instead of where it was opened. FILENAME will then be used for all later
  writes.
