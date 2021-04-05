use std::io::{self, stdin, stdout, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const INPUT_BUF_SIZE: usize = 1024;

pub struct Terminal {
    history: Vec<String>,
    history_item: usize, // Index into history
}

impl Terminal {
    pub fn new() -> Self {
        Terminal {
            history: Vec::with_capacity(10),
            history_item: 0,
        }
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn edit_line(&mut self, prompt: &str, line: &str) -> String {
        let mut stdout = stdout()
            .into_raw_mode()
            .expect("Failed to enable raw mode on std input");

        let mut buf = vec![0 as char; INPUT_BUF_SIZE];
        let mut buf_len = 0;
        let mut cursor_position = 0;

        write!(stdout, "{}", prompt).unwrap();
        stdout.flush().unwrap();

        for c in line.chars() {
            buf[cursor_position] = c;
            cursor_position += 1;
            buf_len += 1;
        }

        write!(stdout, "{}", line).unwrap();
        stdout.flush().unwrap();

        for c in stdin().keys() {
            match c.unwrap() {
                Key::Char(c) => {
                    if (c as u8) == 0x0A || (c as u8) == 0x0D {
                        write!(stdout, "\n\r").unwrap();
                        stdout.flush().unwrap();
                        break;
                    }

                    if cursor_position == buf_len {
                        buf[cursor_position] = c;

                        if buf_len < INPUT_BUF_SIZE {
                            buf_len += 1;
                        }

                        write!(stdout, "{}", c).unwrap();
                    } else {
                        for i in (cursor_position..=buf_len).rev() {
                            if i == 0 {
                                buf[i] = 0 as char;
                            } else {
                                buf[i] = buf[i - 1];
                            }
                        }
                        buf[cursor_position] = c;
                        buf_len += 1;

                        if cursor_position > 0 {
                            write!(stdout, "{}", termion::cursor::Left(cursor_position as u16))
                                .unwrap();
                        }

                        let cursor_offset = if cursor_position == 0 {
                            buf_len - cursor_position
                        } else {
                            buf_len - cursor_position - 1
                        };

                        write!(
                            stdout,
                            "{}{}",
                            buf.iter().collect::<String>(),
                            termion::cursor::Left((cursor_offset) as u16),
                        )
                        .unwrap();
                    }
                    cursor_position += 1;
                }
                Key::Ctrl(c) => {
                    if c == 'c' {
                        buf_len = 0;
                        cursor_position = 0;
                        self.history_item = self.history.len();
                        write!(stdout, "\n\r\u{001b}[2K{}", prompt).unwrap();
                    }
                }
                Key::Left => {
                    if cursor_position > 0 {
                        write!(stdout, "\u{001b}[1D").unwrap();
                        cursor_position -= 1;
                    }
                }
                Key::Right => {
                    if cursor_position < buf_len {
                        write!(stdout, "\u{001b}[1C").unwrap();
                        cursor_position += 1;
                    }
                }
                Key::Backspace => {
                    if buf_len > 0 {
                        if cursor_position == buf_len {
                            buf_len -= 1;
                            cursor_position -= 1;
                            buf[buf_len] = 0 as char;
                            write!(
                                stdout,
                                "{} {}",
                                termion::cursor::Left(1),
                                termion::cursor::Left(1)
                            )
                            .unwrap();
                        } else {
                            for i in cursor_position - 1..buf_len {
                                buf[i] = buf[i + 1]
                            }
                            buf_len -= 1;
                            buf[buf_len] = 0 as char;

                            write!(
                                stdout,
                                "{}{} {}",
                                termion::cursor::Left(cursor_position as u16),
                                buf.iter().collect::<String>(),
                                termion::cursor::Left((buf_len - cursor_position + 2) as u16),
                            )
                            .unwrap();

                            cursor_position -= 1;
                        }
                    }
                }
                Key::Delete => {
                    if buf_len > 0 {
                        if cursor_position == buf_len - 1 {
                            buf[buf_len] = 0 as char;
                            buf_len -= 1;
                            write!(stdout, " {}", termion::cursor::Left(1),).unwrap();
                        } else {
                            for i in cursor_position..buf_len {
                                buf[i] = buf[i + 1]
                            }
                            buf_len -= 1;
                            buf[buf_len] = 0 as char;

                            if cursor_position == 0 {
                                write!(
                                    stdout,
                                    "{} {}",
                                    buf.iter().collect::<String>(),
                                    termion::cursor::Left((buf_len + 1) as u16),
                                )
                                .unwrap();
                            } else {
                                write!(
                                    stdout,
                                    "{}{} {}",
                                    termion::cursor::Left(cursor_position as u16),
                                    buf.iter().collect::<String>(),
                                    termion::cursor::Left((buf_len - cursor_position + 1) as u16),
                                )
                                .unwrap();
                            }
                        }
                    }
                }
                Key::Home => {
                    if cursor_position > 0 {
                        write!(stdout, "{}", termion::cursor::Left(cursor_position as u16))
                            .unwrap();
                        cursor_position = 0;
                    }
                }
                Key::End => {
                    if cursor_position < buf_len {
                        write!(
                            stdout,
                            "{}",
                            termion::cursor::Right((buf_len - cursor_position) as u16)
                        )
                        .unwrap();
                        cursor_position = buf_len;
                    }
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }

        buf[..buf_len].iter().collect()
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn readline(&mut self, prompt: &str) -> String {
        let mut stdout = stdout()
            .into_raw_mode()
            .expect("Failed to enable raw mode on std input");

        let mut buf = vec![0 as char; INPUT_BUF_SIZE];
        let mut buf_len = 0;
        let mut cursor_position = 0;

        write!(stdout, "{}", prompt).unwrap();
        stdout.flush().unwrap();

        for c in stdin().keys() {
            match c.unwrap() {
                Key::Char(c) => {
                    if (c as u8) == 0x0A || (c as u8) == 0x0D {
                        write!(stdout, "\n\r").unwrap();
                        stdout.flush().unwrap();
                        self.history_item = self.history.len();
                        break;
                    }

                    if cursor_position == buf_len {
                        buf[cursor_position] = c;

                        if buf_len < INPUT_BUF_SIZE {
                            buf_len += 1;
                        }

                        write!(stdout, "{}", c).unwrap();
                    } else {
                        for i in (cursor_position..=buf_len).rev() {
                            if i == 0 {
                                buf[i] = 0 as char;
                            } else {
                                buf[i] = buf[i - 1];
                            }
                        }
                        buf[cursor_position] = c;
                        buf_len += 1;

                        if cursor_position > 0 {
                            write!(stdout, "{}", termion::cursor::Left(cursor_position as u16))
                                .unwrap();
                        }

                        let cursor_offset = if cursor_position == 0 {
                            buf_len - cursor_position
                        } else {
                            buf_len - cursor_position - 1
                        };

                        write!(
                            stdout,
                            "{}{}",
                            buf.iter().collect::<String>(),
                            termion::cursor::Left((cursor_offset) as u16),
                        )
                        .unwrap();
                    }
                    cursor_position += 1;
                }
                Key::Ctrl(c) => {
                    if c == 'c' {
                        buf_len = 0;
                        cursor_position = 0;
                        self.history_item = self.history.len();
                        write!(stdout, "\n\r\u{001b}[2K{}", prompt).unwrap();
                    }
                }
                Key::Up => {
                    if self.history_item > 0 {
                        let item = &self.history[self.history_item - 1];
                        write!(stdout, "\r\u{001b}[2K{}{}", prompt, item).unwrap();
                        self.history_item -= 1;
                        buf_len = 0;
                        cursor_position = 0;
                        for c in item.chars() {
                            buf[cursor_position] = c;
                            buf_len += 1;
                            cursor_position += 1;
                        }
                    }
                }
                Key::Down => {
                    if self.history_item + 1 < self.history.len() {
                        let item = &self.history[self.history_item + 1];
                        write!(stdout, "\r\u{001b}[2K{}{}", prompt, item).unwrap();
                        self.history_item += 1;
                        buf_len = 0;
                        cursor_position = 0;
                        for c in item.chars() {
                            buf[cursor_position] = c;
                            buf_len += 1;
                            cursor_position += 1;
                        }
                    } else {
                        buf_len = 0;
                        cursor_position = 0;
                        self.history_item = self.history.len();
                        write!(stdout, "\r\u{001b}[2K{}", prompt).unwrap();
                    }
                }
                Key::Left => {
                    if cursor_position > 0 {
                        write!(stdout, "\u{001b}[1D").unwrap();
                        cursor_position -= 1;
                    }
                }
                Key::Right => {
                    if cursor_position < buf_len {
                        write!(stdout, "\u{001b}[1C").unwrap();
                        cursor_position += 1;
                    }
                }
                Key::Backspace => {
                    if buf_len > 0 {
                        if cursor_position == buf_len {
                            buf_len -= 1;
                            cursor_position -= 1;
                            buf[buf_len] = 0 as char;
                            write!(
                                stdout,
                                "{} {}",
                                termion::cursor::Left(1),
                                termion::cursor::Left(1)
                            )
                            .unwrap();
                        } else {
                            for i in cursor_position - 1..buf_len {
                                buf[i] = buf[i + 1]
                            }
                            buf_len -= 1;
                            buf[buf_len] = 0 as char;

                            write!(
                                stdout,
                                "{}{} {}",
                                termion::cursor::Left(cursor_position as u16),
                                buf.iter().collect::<String>(),
                                termion::cursor::Left((buf_len - cursor_position + 2) as u16),
                            )
                            .unwrap();

                            cursor_position -= 1;
                        }
                    }
                }
                Key::Delete => {
                    if buf_len > 0 {
                        if cursor_position == buf_len - 1 {
                            buf[buf_len] = 0 as char;
                            buf_len -= 1;
                            write!(stdout, " {}", termion::cursor::Left(1),).unwrap();
                        } else {
                            for i in cursor_position..buf_len {
                                buf[i] = buf[i + 1]
                            }
                            buf_len -= 1;
                            buf[buf_len] = 0 as char;

                            if cursor_position == 0 {
                                write!(
                                    stdout,
                                    "{} {}",
                                    buf.iter().collect::<String>(),
                                    termion::cursor::Left((buf_len + 1) as u16),
                                )
                                .unwrap();
                            } else {
                                write!(
                                    stdout,
                                    "{}{} {}",
                                    termion::cursor::Left(cursor_position as u16),
                                    buf.iter().collect::<String>(),
                                    termion::cursor::Left((buf_len - cursor_position + 1) as u16),
                                )
                                .unwrap();
                            }
                        }
                    }
                }
                Key::Home => {
                    if cursor_position > 0 {
                        write!(stdout, "{}", termion::cursor::Left(cursor_position as u16))
                            .unwrap();
                        cursor_position = 0;
                    }
                }
                Key::End => {
                    if cursor_position < buf_len {
                        write!(
                            stdout,
                            "{}",
                            termion::cursor::Right((buf_len - cursor_position) as u16)
                        )
                        .unwrap();
                        cursor_position = buf_len;
                    }
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }

        let line: String = buf[..buf_len].iter().collect();

        self.history.push(line.clone());
        self.history_item += 1;

        line
    }
}

impl io::Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        stdout().flush()
    }
}
