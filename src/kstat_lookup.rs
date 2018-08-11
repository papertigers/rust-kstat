use libc;
use std::ffi::CString;
use std::io;
use std::ptr;

/// Internal struct that represents the `module` and `name` fields in a kstat lookup triplet
pub struct KstatTriplet {
    // To avoid the CString ptr from  dropping in an unsafe block we store it internally
    // and expose a `as_ptr()` method to get back the 'char *'.
    inner: Option<CString>,
}

impl KstatTriplet {
    pub fn new(field: Option<&str>) -> io::Result<Self> {
        match field {
            None => Ok(KstatTriplet { inner: None }),
            Some(f) => {
                let c_string = CString::new(f)?;
                Ok(KstatTriplet {
                    inner: Some(c_string),
                })
            }
        }
    }

    pub fn as_ptr(&self) -> *const libc::c_char {
        match self.inner {
            None => ptr::null(),
            Some(ref s) => s.as_ptr(),
        }
    }
}
