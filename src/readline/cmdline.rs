use alloc::string::String;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::ansi::TerminalExt;
use crate::{Errno, Fd, Result};

/// Represents the command-line.
///
/// This structure represents both the literal characters of the command-line,
/// and its representation on the screen (including the position of the cursor).
pub struct Cmdline {
    /// The file descriptor to which we're writing the command-line.
    fd: Fd,
    /// The command-line buffer.
    cmdline: String,
    /// The index of the cursor within the command line string, in bytes.
    cursor_index: usize,
}

impl Default for Cmdline {
    #[inline]
    fn default() -> Self {
        Self::stdout()
    }
}

impl Cmdline {
    /// Creates a new command-line with the given file descriptor.
    pub const fn from_fd(fd: Fd) -> Self {
        Self {
            cmdline: String::new(),
            cursor_index: 0,
            fd,
        }
    }

    /// Creates a new command-line that writes to the standard output.
    #[inline]
    pub const fn stdout() -> Self {
        Self::from_fd(Fd::STDOUT)
    }

    /// Resets the command-line, removing any input that it might have.
    ///
    /// # Notes
    ///
    /// This function does not take care of the prompt. It only affects the command-line
    /// buffer.
    pub fn clear_buffer(&mut self) {
        // If the command-line is too large, we re-allocate it with something smaller
        // to save memory.
        if self.cmdline.capacity() > 128 {
            self.cmdline = String::new();
        } else {
            self.cmdline.clear();
        }

        self.cursor_index = 0;
    }

    /// Clears the currently displayed line and moves the cursor to the beginning of the line.
    ///
    /// This function takes care of resetting the buffer.
    pub fn clear_line(&mut self) -> Result<()> {
        let width = self.cmdline.width() as u32;

        self.cmdline.clear();
        self.cursor_index = 0;

        self.fd.move_cursor_left(width)?;
        self.fd.clear_line_from_cursor()?;

        Ok(())
    }

    /// Returns the file descriptor that the command-line is using.
    #[inline]
    pub fn fd(&self) -> Fd {
        self.fd
    }

    /// Returns the command-line buffer.
    #[inline]
    pub fn buffer(&self) -> &str {
        &self.cmdline
    }

    /// Returns the character that's just before the cursor.
    #[inline]
    pub fn character_before_cursor(&self) -> Option<char> {
        self.characters_before_cursor().chars().next_back()
    }

    /// Returns the characters that are before the cursor.
    #[inline]
    pub fn characters_before_cursor(&self) -> &str {
        unsafe { self.cmdline.get_unchecked(..self.cursor_index) }
    }

    /// Returns the character that's just after the cursor.
    #[inline]
    pub fn character_after_cursor(&self) -> Option<char> {
        self.characters_after_cursor().chars().next()
    }

    /// Returns the characters that are after the cursor.
    #[inline]
    pub fn characters_after_cursor(&self) -> &str {
        unsafe { self.cmdline.get_unchecked(self.cursor_index..) }
    }

    /// Inserts a character at the cursor's position.
    ///
    /// # Remarks
    ///
    /// This function assumes that the added characters are printable.
    pub fn insert_at_cursor(&mut self, characters: &str) -> Result<()> {
        // Insert the characters into the command line buffer.
        self.cmdline
            .try_reserve(characters.len())
            .map_err(|_| Errno::NOMEM)?;
        self.cmdline.insert_str(self.cursor_index, characters);

        // Clear the line from the cursor's position.
        self.fd.clear_line_from_cursor()?;

        // Write the modified command line. This will move the cursor of the terminal.
        let rest = self.characters_after_cursor();
        let rest_width = rest.width() as u32;
        self.fd.write_all(rest.as_bytes())?;

        // Place the cursor at the right position.
        self.cursor_index += characters.len();
        self.fd
            .move_cursor_left(rest_width - characters.width() as u32)?;

        Ok(())
    }

    /// Erases one character at the cursor's position.
    #[inline]
    pub fn erase_one_at_cursor(&mut self) -> Result<()> {
        if let Some(c) = self.character_before_cursor() {
            // Update the buffer.
            let start = self.cursor_index - c.len_utf8();
            self.cmdline.drain(start..self.cursor_index);
            self.cursor_index = start;

            // Move the cursor back and clear the line from that position.
            self.fd
                .move_cursor_left(c.width().unwrap_or_default() as u32)?;
            self.fd.clear_line_from_cursor()?;

            // Write the modified command line.
            let rest = self.characters_after_cursor();
            let rest_width = rest.width() as u32;
            self.fd.write_all(rest.as_bytes())?;

            // Replace the cursor at the right position.
            self.fd.move_cursor_left(rest_width)?;
        }

        Ok(())
    }

    /// Moves the cursor to the left.
    #[inline]
    pub fn move_cursor_left(&mut self) -> Result<()> {
        if let Some(c) = self.character_before_cursor() {
            self.cursor_index -= c.len_utf8();
            self.fd
                .move_cursor_left(c.width().unwrap_or_default() as u32)?;
        }

        Ok(())
    }

    /// Moves the cursor to the right.
    #[inline]
    pub fn move_cursor_right(&mut self) -> Result<()> {
        if let Some(c) = self.character_after_cursor() {
            self.cursor_index += c.len_utf8();
            self.fd
                .move_cursor_right(c.width().unwrap_or_default() as u32)?;
        }

        Ok(())
    }
}
