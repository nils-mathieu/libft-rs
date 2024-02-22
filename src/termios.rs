use core::mem::MaybeUninit;

use bitflags::bitflags;

use crate::{Fd, Result};

/// Represents the list of special characters used by the terminal.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SpecialCharacters([libc::cc_t; libc::NCCS]);

impl SpecialCharacters {
    /// Returns the end-of-file character.
    #[inline]
    pub const fn end_of_file(&self) -> u8 {
        self.0[libc::VEOF]
    }

    /// Returns the character caracterizing the end of the line.
    #[inline]
    pub const fn end_of_line(&self) -> u8 {
        self.0[libc::VEOL]
    }

    /// Another end-of-line character.
    #[inline]
    pub const fn end_of_line2(&self) -> u8 {
        self.0[libc::VEOL2]
    }

    /// The character used to erase the last character.
    #[inline]
    pub const fn erase(&self) -> u8 {
        self.0[libc::VERASE]
    }

    /// The interrupt character.
    ///
    /// Normally triggers a `SIGINT`.
    #[inline]
    pub const fn intr(&self) -> u8 {
        self.0[libc::VINTR]
    }

    /// The character used to kill the current line.
    #[inline]
    pub const fn kill(&self) -> u8 {
        self.0[libc::VKILL]
    }

    /// The character used to quit the current process.
    ///
    /// Normally triggers a `SIGQUIT`.
    #[inline]
    pub const fn quit(&self) -> u8 {
        self.0[libc::VQUIT]
    }

    /// The character used to erase a whole word.
    #[inline]
    pub const fn word_erase(&self) -> u8 {
        self.0[libc::VWERASE]
    }
}

/// Describes *when* to apply a new [`Termios`] configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetAt {
    /// Right now.
    Now = libc::TCSANOW as isize,
    /// Only when the output buffer has been transmitted.
    Drain = libc::TCSADRAIN as isize,
    /// Only when both the input and output buffers have been transmitted.
    Flush = libc::TCSAFLUSH as isize,
}

/// Represents the Terminal I/O structure.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Termios(libc::termios);

impl Termios {
    /// Gets the [`Termios`] structure of the provided file descriptor.
    #[inline]
    pub fn get(fd: Fd) -> Result<Self> {
        let mut termios = MaybeUninit::uninit();
        let ret = unsafe { libc::tcgetattr(fd.to_raw(), termios.as_mut_ptr()) };

        if ret == -1 {
            Err(crate::Errno::last())
        } else {
            Ok(unsafe { Self(termios.assume_init()) })
        }
    }

    /// Sets the [`Termios`] structure of the provided file descriptor.
    #[inline]
    pub fn set(&self, fd: Fd, when: SetAt) -> Result<()> {
        let ret = unsafe { libc::tcsetattr(fd.to_raw(), when as _, &self.0) };

        if ret == -1 {
            Err(crate::Errno::last())
        } else {
            Ok(())
        }
    }

    /// Returns the input flags.
    #[inline]
    pub fn input(&self) -> InputFlags {
        InputFlags::from_bits_retain(self.0.c_iflag)
    }

    /// Returns the output flags.
    #[inline]
    pub fn output(&self) -> OutputFlags {
        OutputFlags::from_bits_retain(self.0.c_oflag)
    }

    /// Returns the control flags.
    #[inline]
    pub fn control(&self) -> ControlFlags {
        ControlFlags::from_bits_retain(self.0.c_cflag)
    }

    /// Returns the local flags.
    #[inline]
    pub fn local(&self) -> LocalFlags {
        LocalFlags::from_bits_retain(self.0.c_lflag)
    }

    /// Returns the special characters.
    #[inline]
    pub fn special_characters(&self) -> SpecialCharacters {
        SpecialCharacters(self.0.c_cc)
    }

    /// Returns the input flags.
    #[inline]
    pub fn input_mut(&mut self) -> &mut InputFlags {
        unsafe { &mut *(self as *mut Self as *mut InputFlags) }
    }

    /// Returns the output flags.
    #[inline]
    pub fn output_mut(&mut self) -> &mut OutputFlags {
        unsafe { &mut *(self as *mut Self as *mut OutputFlags) }
    }

    /// Returns the control flags.
    #[inline]
    pub fn control_mut(&mut self) -> &mut ControlFlags {
        unsafe { &mut *(self as *mut Self as *mut ControlFlags) }
    }

    /// Returns the local flags.
    #[inline]
    pub fn local_mut(&mut self) -> &mut LocalFlags {
        unsafe { &mut *(self as *mut Self as *mut LocalFlags) }
    }

    /// Returns the special characters.
    #[inline]
    pub fn special_characters_mut(&mut self) -> &mut SpecialCharacters {
        unsafe { &mut *(self as *mut Self as *mut SpecialCharacters) }
    }
}

bitflags! {
    /// The flags that are used to control the input of a terminal.
    ///
    /// # Notes on BREAK
    ///
    /// 1. If `IGNORE_BREAK` is set, then BREAK is ignored.
    ///
    /// 2. If `BREAK_INT` is set, then BREAK flushes the input and output streams and sends an
    ///    interrupt signal to the foreground process group.
    ///
    /// 3. If `PARITY_MARK` is set, then BREAK sends `0xFF 0x00 0x00`.
    ///
    /// 4. Otherwise, `0x00` is sent.
    ///
    /// # Parity and framing Errors
    ///
    /// 1. If `IGNORE_PARITY` is set, then framing and parity errors are ignored.
    ///
    /// 2. If `MARK_PARITY` is set, then framing and parity errors are marked with `0xFF 0x00`.
    ///
    /// 3. Othewise, framing and parity errors are replaced with `0x00`.
    #[derive(Default, Clone, Copy, Debug)]
    #[repr(transparent)]
    pub struct InputFlags: libc::tcflag_t {
        /// Ignore BREAK condition on input.
        const IGNORE_BREAK = libc::IGNBRK;
        /// Causes BREAK to flush the input and output queues and causes a `SIGINT` to be sent
        /// to the foreground process group.
        ///
        /// See the [struct-level] documentation for more information.
        const BREAK_INT = libc::BRKINT;
        /// Ignore framing and parity errors.
        const IGNORE_PARITY = libc::IGNPAR;
        /// Mark parity and framing errors.
        ///
        /// Errors are preceded by `0xFF 0x00`.
        ///
        /// To avoid confusing a valid `0xFF` character with a parity error, when this
        /// flag is set, two `0xFF` characters are sent to represent a valid `0xFF` character.
        ///
        /// When unset, parity and framing errors are represented by a `0x00` character.
        const PARITY_MARK = libc::PARMRK;
        /// Automatically sets all eighth bits to 0.
        const STRIP = libc::ISTRIP;
        /// Translates new-line characters to carriage return characters.
        const NL_TO_CR = libc::INLCR;
        /// Ignore carriage returns.
        const IGNORE_CR = libc::IGNCR;
        /// Translate carriage returns to new-line characters.
        const CR_TO_NL = libc::ICRNL;
    }
}

bitflags! {
    /// The flags that are used to control the output of a terminal.
    #[derive(Default, Clone, Copy, Debug)]
    #[repr(transparent)]
    pub struct OutputFlags: libc::tcflag_t {
        /// Enable implementation-defined output processing.
        const POST_PROCESS_OUTPUT = libc::OPOST;
        /// Automatically add a carriage-return character before every new-line.
        const AUTO_CR = libc::ONLCR;
        /// Translate new-line characters to carriage return characters.
        const NL_TO_CR = libc::OCRNL;
        /// Don't output a carriage return on column 0.
        const NO_CR_ON_COL0 = libc::ONOCR;
        /// Assume that new-lines do the carriage-return function.
        const NL_DO_CR = libc::ONLRET;
    }
}

bitflags! {
    /// The flags that are used to control the terminal itself.
    #[derive(Default, Clone, Copy, Debug)]
    #[repr(transparent)]
    pub struct ControlFlags: libc::tcflag_t {
        /// Enable receiver.
        const READ = libc::CREAD;
        /// Enable parity generation on output and parity checking for input.
        const PARITY = libc::PARENB;
        /// Use odd parity instead of even parity.
        const PARITY_ODD = libc::PARODD;
    }
}

bitflags! {
    /// The flags that are used to control the local mode of a terminal.
    #[derive(Default, Clone, Copy, Debug)]
    #[repr(transparent)]
    pub struct LocalFlags: libc::tcflag_t {
        /// When signal characters are received, automatically send the corresponding signal.
        const SIGNAL = libc::ISIG;
        /// Enable cannonical mode.
        const CANONICAL = libc::ICANON;
        /// Echo input characters to the output.
        const ECHO = libc::ECHO;
        /// In canonical mode, erase the last character in the line when the backspace character
        /// is received.
        const ERASE = libc::ECHOE;
        /// In canonical mode, erase the last character in the line when the backspace character
        /// is received.
        const KILL = libc::ECHOKE;
        /// Echoes the newline character even if `ECHO` is not set.
        const ECHO_NL = libc::ECHONL;
        /// Disables flusing the input/output when signal characters are received.
        const NO_FLUSH = libc::NOFLSH;
    }
}
