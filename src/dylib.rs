//! Things related to the `dlopen` system call.

use core::mem::ManuallyDrop;

use libc::c_void;

use crate::{CharStar, Errno, Result};

/// A loaded dynamic library.
pub struct Dylib {
    /// The library handle delivered by the system.
    handle: *mut c_void,
}

unsafe impl Send for Dylib {}
unsafe impl Sync for Dylib {}

impl Dylib {
    /// Opens a new dynamic library in the current process's memory.
    ///
    /// This function (as opposed to [`Dylib::new`]) does not resolve any symbols in the library
    /// immediately. Instead, symbols are resolved as they are used. This can be useful for
    /// libraries with many symbols, most of which are not used.
    ///
    /// # Safety
    ///
    /// Opening a dynamic library is inherently unsafe, as it can lead to undefined behavior if the
    /// library is not properly constructed. This function is marked as `unsafe` to reflect that
    /// fact. The caller must ensure that the library's constructor won't cause undefined behavior.
    #[inline]
    pub unsafe fn new_lazy(filename: &CharStar) -> Result<Self> {
        let handle = unsafe { libc::dlopen(filename.as_ptr(), libc::RTLD_LAZY) };
        if handle.is_null() {
            Err(Errno::last())
        } else {
            Ok(Self { handle })
        }
    }

    /// Opens a new dynamic library in the current process's memory.
    ///
    /// This function (as opposed to [`Dylib::new_lazy`]) resolves all symbols in the library
    /// immediately. When this function returns, all the symbols are available for use.
    ///
    /// # Safety
    ///
    /// Opening a dynamic library is inherently unsafe, as it can lead to undefined behavior if the
    /// library is not properly constructed. This function is marked as `unsafe` to reflect that
    /// fact. The caller must ensure that the library's constructor won't cause undefined behavior.
    #[inline]
    pub unsafe fn new(filename: &CharStar) -> Result<Self> {
        let handle = unsafe { libc::dlopen(filename.as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            Err(Errno::last())
        } else {
            Ok(Self { handle })
        }
    }

    /// Determines whether the library with the provided file name is
    /// currently loaded.
    #[inline]
    pub fn is_loaded(filename: &CharStar) -> bool {
        let handle = unsafe { libc::dlopen(filename.as_ptr(), libc::RTLD_NOLOAD) };
        !handle.is_null()
    }

    /// Attempts to close the library.
    #[inline]
    pub fn close(self) -> Result<()> {
        let this = ManuallyDrop::new(self);
        let ret = unsafe { libc::dlclose(this.handle) };

        if ret == 0 {
            Ok(())
        } else {
            Err(Errno::last())
        }
    }

    /// Resolves a symbol in the lirary.
    ///
    /// The returned pointer is the address of the symbol within the address
    /// space of the current process.
    ///
    /// If the function returns [`Ok(_)`], then the symbol was found and
    /// it is guaranteed not be null.
    #[inline]
    pub fn raw_symbol(&self, name: &CharStar) -> Result<*const c_void> {
        let symbol = unsafe { libc::dlsym(self.handle, name.as_ptr()) };
        if symbol.is_null() {
            Err(Errno::last())
        } else {
            Ok(symbol)
        }
    }

    /// Resolves a symbol in the library.
    ///
    /// # Panics
    ///
    /// This function panics at compile-time if the size of `T` is not equal to the size
    /// of a pointer.
    ///
    /// # Safety
    ///
    /// The returned instance must:
    ///
    /// 1. Not outlive the library it was resolved from.
    ///
    /// 2. Be of the correct type.
    #[inline]
    pub unsafe fn symbol<T>(&self, name: &CharStar) -> Result<T> {
        assert_eq!(
            core::mem::size_of::<T>(),
            core::mem::size_of::<*const c_void>()
        );

        let symbol = self.raw_symbol(name)?;
        Ok(unsafe { core::mem::transmute_copy(&symbol) })
    }
}

impl Drop for Dylib {
    #[inline]
    fn drop(&mut self) {
        unsafe { libc::dlclose(self.handle) };
    }
}
