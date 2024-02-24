use crate::{Fd, Result};

/// A **Select Graphic Rendition** (SGR) code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sgr {
    /// Resets the graphic rendition to the default.
    Normal,
    /// Bold, or increased intensity.
    Bold,
    /// Faint, or decreased intensity.
    Faint,
    /// Italicized.
    Italic,
    /// Underlined.
    Underline,
    /// Blinking.
    SlowBlink,
    /// Rapid blinking.
    RapidBlink,
    /// Inverted colors.
    Invert,
    /// Concealed, or hidden.
    Hide,
    /// Crossed-out.
    CrossedOut,
    /// The default font.
    DefaultFont,
    /// The first alternative font.
    AltFont1,
    /// The second alternative font.
    AltFont2,
    /// The third alternative font.
    AltFont3,
    /// The fourth alternative font.
    AltFont4,
    /// The fifth alternative font.
    AltFont5,
    /// The sixth alternative font.
    AltFont6,
    /// The seventh alternative font.
    AltFont7,
    /// The eighth alternative font.
    AltFont8,
    /// The ninth alternative font.
    AltFont9,
    /// Doubly underlined.
    DoubleUnderline,
    /// Normal intensity (nor bold, nor faint).
    ClearIntensity,
    /// Not italicized.
    ClearItalic,
    /// Not underlined.
    ClearUnderline,
    /// Not blinking.
    ClearBlink,
    /// Not inverted.
    ClearInvert,
    /// Not concealed.
    ClearHide,
    /// Not crossed-out.
    ClearCrossedOut,
    /// The black foreground color.
    FgBlack,
    /// The red foreground color.
    FgRed,
    /// The green foreground color.
    FgGreen,
    /// The yellow foreground color.
    FgYellow,
    /// The blue foreground color.
    FgBlue,
    /// The magenta foreground color.
    FgMagenta,
    /// The cyan foreground color.
    FgCyan,
    /// The white foreground color.
    FgWhite,
    /// A RGB foreground color.
    FgRgb(u8, u8, u8),
    /// The default foreground color.
    FgDefault,
    /// The black background color.
    BgBlack,
    /// The red background color.
    BgRed,
    /// The green background color.
    BgGreen,
    /// The yellow background color.
    BgYellow,
    /// The blue background color.
    BgBlue,
    /// The magenta background color.
    BgMagenta,
    /// The cyan background color.
    BgCyan,
    /// The white background color.
    BgWhite,
    /// A RGB background color.
    BgRgb(u8, u8, u8),
    /// The default background color.
    BgDefault,
    /// The bright black foreground color.
    FgBrightBlack,
    /// The bright red foreground color.
    FgBrightRed,
    /// The bright green foreground color.
    FgBrightGreen,
    /// The bright yellow foreground color.
    FgBrightYellow,
    /// The bright blue foreground color.
    FgBrightBlue,
    /// The bright magenta foreground color.
    FgBrightMagenta,
    /// The bright cyan foreground color.
    FgBrightCyan,
    /// The bright white foreground color.
    FgBrightWhite,
    /// The bright black background color.
    BgBrightBlack,
    /// The bright red background color.
    BgBrightRed,
    /// The bright green background color.
    BgBrightGreen,
    /// The bright yellow background color.
    BgBrightYellow,
    /// The bright blue background color.
    BgBrightBlue,
    /// The bright magenta background color.
    BgBrightMagenta,
    /// The bright cyan background color.
    BgBrightCyan,
    /// The bright white background color.
    BgBrightWhite,
}

impl Sgr {
    /// Writes the SGR code to the given file descriptor.
    pub fn write(self, fd: Fd) -> Result<()> {
        match self {
            Sgr::Normal => write!(fd, "0"),
            Sgr::Bold => write!(fd, "1"),
            Sgr::Faint => write!(fd, "2"),
            Sgr::Italic => write!(fd, "3"),
            Sgr::Underline => write!(fd, "4"),
            Sgr::SlowBlink => write!(fd, "5"),
            Sgr::RapidBlink => write!(fd, "6"),
            Sgr::Invert => write!(fd, "7"),
            Sgr::Hide => write!(fd, "8"),
            Sgr::CrossedOut => write!(fd, "9"),
            Sgr::DefaultFont => write!(fd, "10"),
            Sgr::AltFont1 => write!(fd, "11"),
            Sgr::AltFont2 => write!(fd, "12"),
            Sgr::AltFont3 => write!(fd, "13"),
            Sgr::AltFont4 => write!(fd, "14"),
            Sgr::AltFont5 => write!(fd, "15"),
            Sgr::AltFont6 => write!(fd, "16"),
            Sgr::AltFont7 => write!(fd, "17"),
            Sgr::AltFont8 => write!(fd, "18"),
            Sgr::AltFont9 => write!(fd, "19"),
            Sgr::DoubleUnderline => write!(fd, "21"),
            Sgr::ClearIntensity => write!(fd, "22"),
            Sgr::ClearItalic => write!(fd, "23"),
            Sgr::ClearUnderline => write!(fd, "24"),
            Sgr::ClearBlink => write!(fd, "25"),
            Sgr::ClearInvert => write!(fd, "27"),
            Sgr::ClearHide => write!(fd, "28"),
            Sgr::ClearCrossedOut => write!(fd, "29"),
            Sgr::FgBlack => write!(fd, "30"),
            Sgr::FgRed => write!(fd, "31"),
            Sgr::FgGreen => write!(fd, "32"),
            Sgr::FgYellow => write!(fd, "33"),
            Sgr::FgBlue => write!(fd, "34"),
            Sgr::FgMagenta => write!(fd, "35"),
            Sgr::FgCyan => write!(fd, "36"),
            Sgr::FgWhite => write!(fd, "37"),
            Sgr::FgRgb(r, g, b) => write!(fd, "38;2;{};{};{}", r, g, b),
            Sgr::FgDefault => write!(fd, "39"),
            Sgr::BgBlack => write!(fd, "40"),
            Sgr::BgRed => write!(fd, "41"),
            Sgr::BgGreen => write!(fd, "42"),
            Sgr::BgYellow => write!(fd, "43"),
            Sgr::BgBlue => write!(fd, "44"),
            Sgr::BgMagenta => write!(fd, "45"),
            Sgr::BgCyan => write!(fd, "46"),
            Sgr::BgWhite => write!(fd, "47"),
            Sgr::BgRgb(r, g, b) => write!(fd, "48;2;{};{};{}", r, g, b),
            Sgr::BgDefault => write!(fd, "49"),
            Sgr::FgBrightBlack => write!(fd, "90"),
            Sgr::FgBrightRed => write!(fd, "91"),
            Sgr::FgBrightGreen => write!(fd, "92"),
            Sgr::FgBrightYellow => write!(fd, "93"),
            Sgr::FgBrightBlue => write!(fd, "94"),
            Sgr::FgBrightMagenta => write!(fd, "95"),
            Sgr::FgBrightCyan => write!(fd, "96"),
            Sgr::FgBrightWhite => write!(fd, "97"),
            Sgr::BgBrightBlack => write!(fd, "100"),
            Sgr::BgBrightRed => write!(fd, "101"),
            Sgr::BgBrightGreen => write!(fd, "102"),
            Sgr::BgBrightYellow => write!(fd, "103"),
            Sgr::BgBrightBlue => write!(fd, "104"),
            Sgr::BgBrightMagenta => write!(fd, "105"),
            Sgr::BgBrightCyan => write!(fd, "106"),
            Sgr::BgBrightWhite => write!(fd, "107"),
        }
    }
}

/// An extension trait for [`Fd`] that allows controlling a ANSI-code based terminal.
pub trait TerminalExt {
    /// Moves the up cursor by `count` cells.
    fn move_cursor_up(self, count: u32) -> Result<()>;

    /// Moves the down cursor by `count` cells.
    fn move_cursor_down(self, count: u32) -> Result<()>;

    /// Moves the left cursor by `count` cells.
    fn move_cursor_left(self, count: u32) -> Result<()>;

    /// Moves the right cursor by `count` cells.
    fn move_cursor_right(self, count: u32) -> Result<()>;

    /// Moves the cursor to the beginning of the `n`-th line down.
    fn move_cursor_line_down(self, count: u32) -> Result<()>;

    /// Moves the cursor to the beginning of the `n`-th line up.
    fn move_cursor_line_up(self, count: u32) -> Result<()>;

    /// Move the cursor to column `x`.
    fn set_cursor_x(self, x: u32) -> Result<()>;

    /// Sets the position of the cursor to `x` and `y`.
    ///
    /// Note that values are 1-based.
    fn set_cursor_position(self, x: u32, y: u32) -> Result<()>;

    /// Clears the terminal from the cursor to the end of the screen.
    fn clear_from_cursor(self) -> Result<()>;

    /// Clears the terminal from the cursor to the beginning of the screen.
    fn clear_to_cursor(self) -> Result<()>;

    /// Clears the entire terminal.
    fn clear_screen(self) -> Result<()>;

    /// Clears the screen and resets the scrollback buffer.
    fn clear_screen_and_scrollback_buffer(self) -> Result<()>;

    /// Clears the line from the cursor to the end of the line.
    ///
    /// Note that this does not modify the position of the cursor.
    fn clear_line_from_cursor(self) -> Result<()>;

    /// Clears the line from the cursor to the beginning of the line.
    ///
    /// Note that this does not modify the position of the cursor.
    fn clear_line_to_cursor(self) -> Result<()>;

    /// Clears the entire line.
    ///
    /// Note that this does not modify the position of the cursor.
    fn clear_line(self) -> Result<()>;

    /// Scrolls a complete page up.
    fn scroll_page_up(self) -> Result<()>;

    /// Scrolls a complete page down.
    fn scroll_page_down(self) -> Result<()>;

    /// Modify the appearance of the text displayed.
    fn select_graphic_rendition(self, renditions: &[Sgr]) -> Result<()>;
}

impl TerminalExt for Fd {
    #[inline]
    fn move_cursor_up(self, count: u32) -> Result<()> {
        if count != 0 {
            write!(self, "\x1b[{}A", count)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn move_cursor_down(self, count: u32) -> Result<()> {
        if count != 0 {
            write!(self, "\x1b[{}B", count)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn move_cursor_left(self, count: u32) -> Result<()> {
        if count != 0 {
            write!(self, "\x1b[{}D", count)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn move_cursor_right(self, count: u32) -> Result<()> {
        if count != 0 {
            write!(self, "\x1b[{}C", count)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn move_cursor_line_down(self, count: u32) -> Result<()> {
        if count != 0 {
            write!(self, "\x1b[{}E", count)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn move_cursor_line_up(self, count: u32) -> Result<()> {
        if count != 0 {
            write!(self, "\x1b[{}F", count)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn set_cursor_x(self, x: u32) -> Result<()> {
        write!(self, "\x1b[{}G", x)
    }

    #[inline]
    fn set_cursor_position(self, x: u32, y: u32) -> Result<()> {
        write!(self, "\x1b[{};{}H", y, x)
    }

    #[inline]
    fn clear_from_cursor(self) -> Result<()> {
        self.write_all(b"\x1b[0J")
    }

    #[inline]
    fn clear_to_cursor(self) -> Result<()> {
        self.write_all(b"\x1b[1J")
    }

    #[inline]
    fn clear_screen(self) -> Result<()> {
        self.write_all(b"\x1b[2J")
    }

    #[inline]
    fn clear_screen_and_scrollback_buffer(self) -> Result<()> {
        self.write_all(b"\x1b[3J")
    }

    #[inline]
    fn clear_line_from_cursor(self) -> Result<()> {
        self.write_all(b"\x1b[0K")
    }

    #[inline]
    fn clear_line_to_cursor(self) -> Result<()> {
        self.write_all(b"\x1b[1K")
    }

    #[inline]
    fn clear_line(self) -> Result<()> {
        self.write_all(b"\x1b[2K")
    }

    #[inline]
    fn scroll_page_up(self) -> Result<()> {
        write!(self, "\x1b[S")
    }

    #[inline]
    fn scroll_page_down(self) -> Result<()> {
        write!(self, "\x1b[T")
    }

    #[inline]
    fn select_graphic_rendition(self, renditions: &[Sgr]) -> Result<()> {
        match renditions {
            [] => Ok(()),
            [first, rest @ ..] => {
                self.write_all(b"\x1b[")?;
                first.write(self)?;
                for rendition in rest {
                    self.write_all(b";")?;
                    rendition.write(self)?;
                }
                self.write_all(b"m")?;
                Ok(())
            }
        }
    }
}
