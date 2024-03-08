use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::Cmdline;
use crate::malloc::OutOfMemory;
use crate::readline::{read_non_canonical, Event, Flow};
use crate::utils::box_str;
use crate::{Errno, Fd, SafeStringExt, SafeVecExt};

/// The configuration of the [`readline`] function.
pub struct Readline {
    /// The file descriptor that will be used to read from the terminal.
    input: Fd,
    /// The prompt that is displayed when the command-line is reset.
    prompt: String,
    /// The history available to the command-line.
    history: Vec<Box<str>>,
    /// The state of the command-line.
    cmdline: Cmdline,
}

impl Readline {
    /// Creates a new instance of [`Readline`].
    pub const fn new() -> Self {
        Self {
            input: Fd::STDIN,
            prompt: String::new(),
            history: Vec::new(),
            cmdline: Cmdline::stdout(),
        }
    }

    /// Sets the input file descriptor to use.
    #[inline]
    pub fn set_input_fd(&mut self, input: Fd) {
        self.input = input;
    }

    /// Sets the prompt to display.
    pub fn set_prompt(&mut self, prompt: &str) -> Result<(), OutOfMemory> {
        self.prompt.clear();
        self.prompt.reserve_exact(prompt.len());
        self.prompt.push_str(prompt);
        Ok(())
    }

    /// Adds a command-line to the history.
    pub fn history_add(&mut self, cmdline: &str) -> Result<(), OutOfMemory> {
        if self.history.last().map(Box::as_ref) == Some(cmdline) {
            return Ok(());
        }

        self.history.try_push(box_str(cmdline)?)?;
        Ok(())
    }

    /// Adds the current command-line buffer to the history.
    pub fn history_add_buffer(&mut self) -> Result<(), OutOfMemory> {
        if self.history.last().map(Box::as_ref) == Some(self.cmdline.buffer()) {
            return Ok(());
        }

        self.history.try_push(box_str(self.cmdline.buffer())?)?;
        Ok(())
    }

    /// Clears the history of the command-line.
    #[inline]
    pub fn history_clear(&mut self) {
        self.history.clear();
    }

    /// Returns the command-line buffer.
    #[inline]
    pub fn buffer(&self) -> &str {
        self.cmdline.buffer()
    }

    /// Reads the user input from the terminal.
    ///
    /// # Returns
    ///
    /// `true` is returned actually uses the **Enter** key to submit the input.
    ///
    /// `false` is returned otherwise, for example because the end-of-file character
    /// has been received.
    #[inline]
    pub fn read(&mut self) -> Result<bool, Errno> {
        macro_rules! ftry {
            ($e:expr) => {
                match $e {
                    Ok(ok) => ok,
                    Err(err) => return Flow::Break(Err(err.into())),
                }
            };
        }

        let mut history_index = usize::MAX;
        let mut history_saved_buffer = String::new();

        self.cmdline.clear_buffer();
        self.cmdline.fd().write_all(self.prompt.as_bytes())?;

        read_non_canonical(self.input, |event| match event {
            Event::Enter => {
                if self.cmdline.buffer().is_empty() {
                    ftry!(self.cmdline.fd().write_all(b"\n"));
                    ftry!(self.cmdline.fd().write_all(self.prompt.as_bytes()));
                    Flow::Continue
                } else {
                    Flow::Break(Ok(true))
                }
            }
            Event::Character(c) => {
                let mut buf = [0u8; 4];
                ftry!(self.cmdline.insert_at_cursor(c.encode_utf8(&mut buf)));
                Flow::Continue
            }
            Event::EndOfFile => Flow::Break(Ok(false)),
            Event::Erase => {
                ftry!(self.cmdline.erase_one_at_cursor());
                Flow::Continue
            }
            Event::Left => {
                ftry!(self.cmdline.move_cursor_left());
                Flow::Continue
            }
            Event::Right => {
                ftry!(self.cmdline.move_cursor_right());
                Flow::Continue
            }
            Event::Interrupt => {
                ftry!(self.cmdline.fd().write_all(b"\n"));
                self.cmdline.clear_buffer();
                ftry!(self.cmdline.fd().write_all(self.prompt.as_bytes()));
                Flow::Continue
            }
            Event::Up => {
                if !self.history.is_empty() && history_index != 0 {
                    if history_index == usize::MAX {
                        history_index = self.history.len() - 1;
                        history_saved_buffer.clear();
                        ftry!(history_saved_buffer.try_push_str(self.cmdline.buffer()));
                    } else {
                        history_index -= 1;
                    }

                    ftry!(self.cmdline.clear_line());
                    ftry!(self.cmdline.insert_at_cursor(&self.history[history_index]));
                }

                Flow::Continue
            }
            Event::Down => {
                if !self.history.is_empty() && history_index != usize::MAX {
                    if history_index == self.history.len() - 1 {
                        history_index = usize::MAX;
                        ftry!(self.cmdline.clear_line());
                        ftry!(self.cmdline.insert_at_cursor(&history_saved_buffer));
                    } else {
                        history_index += 1;
                        ftry!(self.cmdline.clear_line());
                        ftry!(self.cmdline.insert_at_cursor(&self.history[history_index]));
                    }
                }

                Flow::Continue
            }
            _ => Flow::Continue,
        })
        .flatten()
    }
}
