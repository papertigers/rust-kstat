use super::ffi;
use std::borrow::Cow;

#[derive(Debug)]
pub enum KstatNamedData {
    DataChar(i8),
    DataInt32(i32),
    DataUInt32(u32),
    DataInt64(i64),
    DataUInt64(u64),
    DataString(String),
}

#[derive(Debug)]
pub(crate) struct KstatNamed {
    inner: *const ffi::kstat_named_t,
}

impl KstatNamed {
    pub fn new(ptr: *const ffi::kstat_named_t) -> Self {
        KstatNamed { inner: ptr }
    }

    pub fn name(&self) -> Cow<str> {
        unsafe { (*self.inner).get_name() }
    }

    fn get_data_type(&self) -> u8 {
        unsafe { (*self.inner).data_type }
    }

    pub fn read(&self) -> (String, KstatNamedData) {
        (self.name().into_owned(), KstatNamedData::from(&*self))
    }
}

impl<'a> From<&'a KstatNamed> for KstatNamedData {
    fn from(t: &'a KstatNamed) -> Self {
        match t.get_data_type() {
            ffi::KSTAT_DATA_CHAR => KstatNamedData::DataChar(unsafe { (*t.inner).value_as_char() }),
            ffi::KSTAT_DATA_INT32 => {
                KstatNamedData::DataInt32(unsafe { (*t.inner).value_as_i32() })
            }
            ffi::KSTAT_DATA_UINT32 => {
                KstatNamedData::DataUInt32(unsafe { (*t.inner).value_as_u32() })
            }
            ffi::KSTAT_DATA_INT64 => {
                KstatNamedData::DataInt64(unsafe { (*t.inner).value_as_i64() })
            }
            ffi::KSTAT_DATA_UINT64 => {
                KstatNamedData::DataUInt64(unsafe { (*t.inner).value_as_u64() })
            }
            ffi::KSTAT_DATA_STRING => {
                KstatNamedData::DataString(unsafe { (*t.inner).value_as_string() })
            }
            _ => panic!("unknown kstat data type"),
        }
    }
}
