use std::fs::{File, OpenOptions};
use std::io::Error;
use std::io::{prelude::*, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::terminal::Terminal;

pub struct Editor {
    filename: Option<PathBuf>,
    newline_seq: &'static str,
    terminal: Terminal,
    contents: Vec<String>,
    curr_line: u32,
}

impl Editor {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Editor, Error> {
        if !path.as_ref().exists() {
            let mut editor = Self::new_empty();
            editor.filename = Some(path.as_ref().to_owned());
            return Ok(editor);
        }

        let mut the_file = OpenOptions::new().read(true).write(true).open(&path)?;

        let mut file_contents = String::new();
        the_file.read_to_string(&mut file_contents)?;
        let newline_char = if file_contents.contains('\n') {
            "\n"
        } else {
            "\r\n"
        };

        Ok(Editor {
            filename: Some(path.as_ref().to_owned()),
            newline_seq: newline_char,
            terminal: Terminal::new(),
            contents: file_contents
                .split(newline_char)
                .map(|s| s.to_owned())
                .collect(),
            curr_line: 0,
        })
    }

    pub fn new_empty() -> Editor {
        Editor {
            filename: None,
            newline_seq: "\n",
            terminal: Terminal::new(),
            contents: Vec::with_capacity(10),
            curr_line: 0,
        }
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
                "d" => self.delete_line(),
                "e" => self.edit_mode(),
                "f" => self.find_next(&cmd[1..]),
                "F" => self.find_prev(&cmd[1..]),
                "i" => self.insert_down(),
                "I" => self.insert_up(),
                "m" => self.metadata(),
                "q" => return,
                "p" => self.print_line(&cmd[1..]),
                "w" => self.save(&cmd[1..]),
                _ => {
                    if let Ok(line) = cmd[0].parse::<u32>() {
                        self.curr_line = if line == 0 { 0 } else { line - 1 };

                        if self.curr_line >= self.contents.len() as u32 {
                            self.curr_line = self.contents.len() as u32 - 1;
                        }
                    }
                }
            }
        }
    }

    fn read_cmd(&mut self) -> String {
        self.terminal
            .readline(&format!("{} > ", self.curr_line + 1))
    }

    fn print_help(&mut self) {
        println!("         NUM - Set current line");
        println!("           ? - Print this help");
        println!("     c [NUM] - Print context, defaults to 2 lines");
        println!("           d - Delete current line");
        println!("           e - Edit current line");
        println!("    f [TEXT] - Find text below current line");
        println!("    F [TEXT] - Find text above current line");
        println!("           i - Insert new line below current line");
        println!("           I - Insert new line above current line");
        println!("           m - Print editor data");
        println!("           q - Quit");
        println!("     p [NUM] - Print current line. If given a number, will set the current line and print it");
        println!("w [FILENAME] - Write file to FILENAME or opened file location");
    }

    fn print_curr_line(&self) {
        println!("{}", self.contents[self.curr_line as usize]);
    }

    fn print_curr_line_with_num(&self) {
        println!(
            "{}: {}",
            self.curr_line + 1,
            self.contents[self.curr_line as usize]
        );
    }

    fn print_line(&mut self, args: &[&str]) {
        if !args.is_empty() {
            let new_line = args[0].parse().unwrap_or(self.curr_line);
            self.curr_line = new_line;
        }

        if self.curr_line >= self.contents.len() as u32 {
            self.curr_line = self.contents.len() as u32 - 1;
        }

        self.print_curr_line();
    }

    fn edit_mode(&mut self) {
        let edited_line = self.terminal.edit_line(
            &format!("{} # ", self.curr_line + 1),
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

    fn save(&mut self, args: &[&str]) {
        if args.is_empty() {
            match &self.filename {
                Some(f) => self.save_file(&f),
                None => println!("No filename given"),
            }
        } else if let Ok(p) = PathBuf::from_str(args[0]) {
            self.save_file(&p);
            self.filename = Some(p);
        } else {
            println!("Invalid file name");
        }
    }

    fn save_file<P: AsRef<Path>>(&self, path: P) {
        let mut the_file = match File::create(&path) {
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
        match &self.filename {
            Some(f) => println!("File: {:?}", f),
            None => println!("File: -"),
        };
        println!("Lines: {}", self.contents.len());
        println!("Current Line: {}", self.curr_line + 1);
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
            println!("{}: {}", line_num + 1, self.contents[line_num]);
        }

        self.print_curr_line_with_num();

        for x in self.curr_line + 1..=context_after {
            let line_num = x as usize;
            println!("{}: {}", line_num + 1, self.contents[line_num]);
        }
    }

    fn delete_line(&mut self) {
        self.contents.remove(self.curr_line as usize);
        if self.curr_line > 0 {
            self.curr_line -= 1;
        }
    }

    fn find_next(&mut self, args: &[&str]) {
        let pattern: String = args.join(" ");

        for (x, line) in self
            .contents
            .iter()
            .skip((self.curr_line as usize) + 1)
            .enumerate()
        {
            if line.contains(&pattern) {
                self.curr_line += x as u32;
                self.print_curr_line_with_num();
                return;
            }
        }

        println!("Pattern '{}' not found.", pattern);
    }

    fn find_prev(&mut self, args: &[&str]) {
        let pattern: String = args.join(" ");

        for (x, line) in self
            .contents
            .iter()
            .rev()
            .skip(self.contents.len() - (self.curr_line as usize))
            .enumerate()
        {
            if line.contains(&pattern) {
                self.curr_line -= (x + 1) as u32;
                self.print_curr_line_with_num();
                return;
            }
        }

        println!("Pattern '{}' not found.", pattern);
    }
}
