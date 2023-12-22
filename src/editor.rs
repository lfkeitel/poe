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
        let newline_char = if file_contents.contains('\r') {
            "\r\n"
        } else {
            "\n"
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
                "c" => self.context_cmd(&cmd[1..]),
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
                "W" => {
                    self.save(&cmd[1..]);
                    return;
                }
                "o" => self.open(&cmd[1..]),
                _ => {
                    if let Ok(line) = cmd[0].parse::<u32>() {
                        self.set_current_line(if line == 0 { 0 } else { line - 1 });
                    }
                    self.print_context(self.curr_line, 2);
                }
            }
        }
    }

    fn set_current_line(&mut self, line: u32) {
        self.curr_line = line;

        if self.curr_line >= self.contents.len() as u32 {
            self.curr_line = self.contents.len() as u32 - 1;
        }
    }

    fn open(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Invalid file name");
            return;
        }

        let path = match PathBuf::from_str(args[0]) {
            Ok(p) => p,
            _ => {
                println!("Invalid file name");
                return;
            }
        };

        if !path.exists() {
            println!("File not found");
            return;
        }

        let mut the_file = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(f) => f,
            _ => {
                println!("Invalid file name");
                return;
            }
        };

        let mut file_contents = String::new();
        if the_file.read_to_string(&mut file_contents).is_err() {
            println!("Error reading file");
            return;
        }
        let newline_char = if file_contents.contains('\n') {
            "\n"
        } else {
            "\r\n"
        };

        self.filename = Some(path);
        self.newline_seq = newline_char;
        self.contents = file_contents
            .split(newline_char)
            .map(|s| s.to_owned())
            .collect();
        self.curr_line = 0;
    }

    fn read_cmd(&mut self) -> String {
        self.terminal
            .readline(&format!("{} > ", self.curr_line + 1))
    }

    fn print_help(&mut self) {
        println!("          NUM - Set current line and print 2 lines of context");
        println!("            ? - Print this help");
        println!("      c [NUM] - Print context, defaults to 2 lines");
        println!("            d - Delete current line");
        println!("            e - Edit current line");
        println!("     f [TEXT] - Find text below current line");
        println!("     F [TEXT] - Find text above current line");
        println!("            i - Insert new line below current line");
        println!("            I - Insert new line above current line");
        println!("            m - Print editor data");
        println!("            q - Quit");
        println!(
            "p [NUM] [CON] - Print current line or line NUM with optional CON lines of context"
        );
        println!(" w [FILENAME] - Write file to FILENAME or opened file location");
        println!(" W [FILENAME] - Write file to FILENAME or opened file location and quit");
        println!(" o [FILENAME] - Open FILENAME");
    }

    fn print_line_with_num(&self, line: u32) {
        println!("{}: {}", line + 1, self.contents[line as usize]);
    }

    fn print_curr_line_with_num(&self) {
        self.print_line_with_num(self.curr_line);
    }

    fn print_line(&mut self, args: &[&str]) {
        let line_num = if args.is_empty() {
            self.curr_line
        } else {
            let new_line = args[0].parse().unwrap_or(self.curr_line);
            if new_line == 0 {
                new_line
            } else {
                new_line - 1
            }
        };

        let context_lines = if args.len() < 2 {
            0
        } else {
            args[1].parse::<i32>().unwrap_or(0)
        };

        self.print_context(line_num, context_lines);
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

    fn context_cmd(&mut self, args: &[&str]) {
        let context_lines = if args.is_empty() {
            2
        } else {
            args[0].parse::<i32>().unwrap_or(2)
        };

        self.print_context(self.curr_line, context_lines);
    }

    fn print_context(&mut self, line_num: u32, context_lines: i32) {
        let context_before = {
            let before = line_num as i32 - context_lines;
            if before < 0 {
                0
            } else {
                before as u32
            }
        };

        let context_after = {
            let after = line_num as i32 + context_lines;
            if after >= self.contents.len() as i32 {
                (after - (after - self.contents.len() as i32) - 1) as u32
            } else {
                after as u32
            }
        };

        for x in context_before..line_num {
            let line_num = x as usize;
            println!("{}: {}", line_num + 1, self.contents[line_num]);
        }

        self.print_line_with_num(line_num);

        for x in line_num + 1..=context_after {
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
                self.curr_line += (x + 1) as u32;
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
