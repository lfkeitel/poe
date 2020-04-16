use std::fs::{File, OpenOptions};
use std::io::Error;
use std::io::{prelude::*, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::terminal::Terminal;

pub struct Editor {
    filename: PathBuf,
    newline_seq: &'static str,
    terminal: Terminal,
    contents: Vec<String>,
    curr_line: u32,
}

impl Editor {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Editor, Error> {
        let mut the_file = OpenOptions::new().read(true).write(true).open(&path)?;

        let mut file_contents = String::new();
        the_file.read_to_string(&mut file_contents)?;
        let newline_char = if file_contents.contains('\n') {
            "\n"
        } else {
            "\r\n"
        };

        Ok(Editor {
            filename: path.as_ref().to_owned(),
            newline_seq: newline_char,
            terminal: Terminal::new_no_history(),
            contents: file_contents
                .split(newline_char)
                .map(|s| s.to_owned())
                .collect(),
            curr_line: 0,
        })
    }

    pub fn run(&mut self) {
        loop {
            let cmd_line = self.read_cmd();
            let cmd: Vec<&str> = cmd_line.split_whitespace().collect();
            if cmd.is_empty() {
                continue;
            }

            match cmd[0] {
                "?" => self.print_help(),
                "c" => self.context(&cmd[1..]),
                "d" => self.delete_line(&cmd[1..]),
                "e" => self.edit_mode(&cmd[1..]),
                "i" => self.insert_down(),
                "I" => self.insert_up(),
                "m" => self.metadata(),
                "q" => return,
                "p" => self.print_line(&cmd[1..]),
                "w" => self.save(&cmd[1..]),
                _ => {
                    if let Ok(line) = cmd[0].parse::<u32>() {
                        self.curr_line = line;

                        if self.curr_line >= self.contents.len() as u32 {
                            self.curr_line = self.contents.len() as u32 - 1;
                        }
                    }
                }
            }
        }
    }

    fn read_cmd(&mut self) -> String {
        self.terminal.readline(&format!("{} > ", self.curr_line))
    }

    fn print_help(&mut self) {
        println!("         NUM - Set current line");
        println!("           ? - Print this help");
        println!("     c [NUM] - Print context, defaults to 2 lines");
        println!("           d - Delete current line");
        println!("           e - Edit current line");
        println!("           i - Insert new line below current line");
        println!("           I - Insert new line above current line");
        println!("           m - Print editor data");
        println!("           q - Quit");
        println!("     p [NUM] - Print current line. If given a number, will set the current line and print it");
        println!("w [FILENAME] - Write file to FILENAME or opened file location");
    }

    fn print_line(&mut self, args: &[&str]) {
        if !args.is_empty() {
            let new_line = args[0].parse().unwrap_or(self.curr_line);
            self.curr_line = new_line;
        }

        if self.curr_line >= self.contents.len() as u32 {
            self.curr_line = self.contents.len() as u32 - 1;
        }

        println!("{}", self.contents[self.curr_line as usize]);
    }

    fn edit_mode(&mut self, args: &[&str]) {
        let edited_line = self.terminal.edit_line(
            &format!("{} # ", self.curr_line),
            &self.contents[self.curr_line as usize],
        );
        self.contents[self.curr_line as usize] = edited_line;
    }

    fn insert_down(&mut self) {
        let new_line = self.terminal.readline("+ ");
        self.curr_line += 1;
        self.contents.insert(self.curr_line as usize, new_line);
    }

    fn insert_up(&mut self) {
        let new_line = self.terminal.readline("+ ");
        self.contents.insert(self.curr_line as usize, new_line);
    }

    fn save(&self, args: &[&str]) {
        if args.is_empty() {
            self.save_file(&self.filename)
        } else if let Ok(p) = PathBuf::from_str(args[1]) {
            self.save_file(&p)
        } else {
            println!("Invalid file name");
        }
    }

    fn save_file<P: AsRef<Path>>(&self, path: P) {
        let mut the_file = match OpenOptions::new().write(true).truncate(true).open(&path) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let mut first = true;

        for line in &self.contents {
            if !first {
                if let Err(e) = the_file.write(self.newline_seq.as_bytes()) {
                    println!("{}", e);
                    return;
                }
            }
            first = false;

            if let Err(e) = the_file.write(line.as_bytes()) {
                println!("{}", e);
                return;
            }
        }

        println!("Saved!");
    }

    fn metadata(&mut self) {
        println!("File: {:?}", self.filename);
        println!("Lines: {}", self.contents.len());
        println!("Current Line: {}", self.curr_line);
    }

    fn context(&mut self, args: &[&str]) {
        let context_lines = if args.is_empty() {
            2
        } else {
            args[0].parse::<i32>().unwrap_or(2)
        };

        let context_before = {
            let before = self.curr_line as i32 - context_lines;
            if before < 0 {
                0
            } else {
                before as u32
            }
        };

        let context_after = {
            let after = self.curr_line as i32 + context_lines;
            if after >= self.contents.len() as i32 {
                (after - (after - self.contents.len() as i32) - 1) as u32
            } else {
                after as u32
            }
        };

        for x in context_before..self.curr_line {
            let line_num = x as usize;
            println!("{}: {}", line_num, self.contents[line_num]);
        }

        println!(
            "{}: {}",
            self.curr_line, self.contents[self.curr_line as usize]
        );

        for x in self.curr_line + 1..=context_after {
            let line_num = x as usize;
            println!("{}: {}", line_num, self.contents[line_num]);
        }
    }

    fn delete_line(&mut self, args: &[&str]) {
        self.contents.remove(self.curr_line as usize);
        if self.curr_line > 0 {
            self.curr_line -= 1;
        }
    }
}
