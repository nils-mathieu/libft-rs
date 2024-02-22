use core::fmt::Write;

/// See [`display_bytes`].
#[repr(transparent)]
pub struct DisplayBytes([u8]);

impl core::fmt::Debug for DisplayBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('\"')?;

        let mut bytes = &self.0;
        while !bytes.is_empty() {
            match core::str::from_utf8(bytes) {
                Ok(s) => {
                    write!(f, "{}", s.escape_debug())?;
                    break;
                }
                Err(e) => {
                    let valid_until = e.valid_up_to();
                    let unknown_from = valid_until + e.error_len().unwrap_or_default();

                    let valid = unsafe { bytes.get_unchecked(..valid_until) };
                    bytes = unsafe { bytes.get_unchecked(unknown_from..) };

                    core::fmt::Display::fmt(
                        unsafe { &core::str::from_utf8_unchecked(valid).escape_debug() },
                        f,
                    )?;
                    f.write_char(char::REPLACEMENT_CHARACTER)?;
                }
            }
        }

        f.write_char('\"')?;
        Ok(())
    }
}

impl core::fmt::Display for DisplayBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut bytes = &self.0;

        while bytes.is_empty() {
            match core::str::from_utf8(bytes) {
                Ok(s) => return s.fmt(f),
                Err(e) => {
                    let valid_until = e.valid_up_to();
                    let unknown_from = valid_until + e.error_len().unwrap_or_default();

                    let valid = unsafe { bytes.get_unchecked(..valid_until) };
                    bytes = unsafe { bytes.get_unchecked(unknown_from..) };

                    f.write_str(unsafe { core::str::from_utf8_unchecked(valid) })?;
                    f.write_char(char::REPLACEMENT_CHARACTER)?;
                }
            }
        }

        Ok(())
    }
}

/// Returns a type that implements [`core::fmt::Display`] which displays
/// the given bytes as a string.
///
/// The bytes are assumed to be valid UTF-8. Any invalid UTF-8 sequence
/// will be replaced with the Unicode replacement character.
///
/// The [`core::fmt::Debug`] implementation is also available.
#[inline]
pub fn display_bytes(b: &[u8]) -> &DisplayBytes {
    unsafe { &*(b as *const [u8] as *const DisplayBytes) }
}
