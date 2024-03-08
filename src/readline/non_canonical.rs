//! Provides a non-canonical mode for the terminal.

use crate::collections::ArrayVec;
use crate::termios::{LocalFlags, SetAt, Termios};
use crate::{Fd, Result};

/// An event that can be read from the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A whole character has been received.
    Character(char),
    /// The end-of-file character has been received.
    ///
    /// This happens when the user presses **Ctrl+D**.
    EndOfFile,
    /// The interrupt character has been received.
    ///
    /// This happens when the user presses **Ctrl+C**.
    Interrupt,
    /// The quit character has been received.
    ///
    /// This happens when the user presses **Ctrl+\**.
    Quit,
    /// The erase character has been received.
    ///
    /// This happens when the user presses **Backspace**.
    Erase,
    /// The kill character has been received.
    ///
    /// This happens when the user presses **Ctrl+U**.
    EraseLine,
    /// Erases the word to the left of the cursor.
    ///
    /// This happens when the user presses **Ctrl+W**.
    EraseWord,
    /// The enter character has been received.
    ///
    /// This happens when the user presses **Enter**.
    Enter,
    /// The enquiry character has been received.
    Enquiry,
    /// The left key.
    Left,
    /// The right key.
    Right,
    /// The up key.
    Up,
    /// The down key.
    Down,
    /// The **Tab** key.
    Tab,
}

/// The return value of the callback function of [`read_non_canonical`].
pub enum Flow<T> {
    /// Continue reading from the terminal.
    Continue,
    /// Stop reading from the terminal.
    Break(T),
}

/// When the terminal has been sent an escape sequence, this state indicates
/// where in the sequence we are.
enum State {
    /// Not in an escape sequence.
    Initial,

    /// The escape character has been received.
    Escaped,

    /// The escape character has been received and the left bracket has been
    /// received.
    LeftBracket,
}

/// Reads the user's input from the terminal in non-canonical mode.
pub fn read_non_canonical<R>(fd: Fd, mut callback: impl FnMut(Event) -> Flow<R>) -> Result<R> {
    // Setup the terminal in non-canonical mode.
    let state = Termios::get(fd)?;
    let _restore = state.guard(fd);
    let mut new_state = state;
    new_state.local_mut().remove(LocalFlags::CANONICAL);
    new_state.local_mut().remove(LocalFlags::ECHO);
    new_state.local_mut().remove(LocalFlags::ECHO_NL);
    new_state.local_mut().remove(LocalFlags::SIGNAL);
    new_state.set(fd, SetAt::Now)?;
    let chars = new_state.special_characters();

    // Initialize the finite state machine.
    let mut state = State::Initial;

    // This buffer is used to store incomplete UTF-8 characters.
    let mut utf8_buf = ArrayVec::<u8, 4>::new();

    // Start the loop and don't restore the terminal until the callback
    // function returns an error.
    loop {
        let byte = match fd.read_one() {
            Ok(Some(b)) => b,
            Ok(None) => chars.eof(),
            Err(e) => return Err(e),
        };

        match state {
            State::Initial => {
                let flow = match byte {
                    _ if byte == chars.eof() => callback(Event::EndOfFile),
                    _ if byte == chars.intr() => callback(Event::Interrupt),
                    _ if byte == chars.erase() => callback(Event::Erase),
                    _ if byte == chars.quit() => callback(Event::Quit),
                    _ if byte == chars.kill() => callback(Event::EraseLine),
                    _ if byte == chars.werase() => callback(Event::EraseWord),
                    b'\t' => callback(Event::Tab),
                    b'\n' => callback(Event::Enter),
                    b'\x05' => callback(Event::Enquiry),
                    b'\x1B' => {
                        state = State::Escaped;
                        Flow::Continue
                    }
                    _ => {
                        utf8_buf.push(byte);
                        if let Ok(s) = core::str::from_utf8(&utf8_buf) {
                            let flow = callback(Event::Character(s.chars().next().unwrap()));
                            utf8_buf.clear();
                            flow
                        } else {
                            Flow::Continue
                        }
                    }
                };

                match flow {
                    Flow::Continue => continue,
                    Flow::Break(r) => break Ok(r),
                }
            }
            State::Escaped => match byte {
                b'[' => state = State::LeftBracket,
                _ => state = State::Initial,
            },
            State::LeftBracket => match byte {
                b'A' => {
                    state = State::Initial;
                    callback(Event::Up);
                }
                b'B' => {
                    state = State::Initial;
                    callback(Event::Down);
                }
                b'C' => {
                    state = State::Initial;
                    callback(Event::Right);
                }
                b'D' => {
                    state = State::Initial;
                    callback(Event::Left);
                }
                _ => state = State::Initial,
            },
        }
    }
}
