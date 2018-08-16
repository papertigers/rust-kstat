use super::ffi;
use libc::c_uchar;

#[derive(Debug)]
pub enum KstatType {
    Raw,
    Named,
    Interrupt,
    IO,
    Timer,
}

impl From<c_uchar> for KstatType {
    fn from(t: c_uchar) -> Self {
        match t {
            ffi::KSTAT_TYPE_RAW => KstatType::Raw,
            ffi::KSTAT_TYPE_NAMED => KstatType::Named,
            ffi::KSTAT_TYPE_INTR => KstatType::Interrupt,
            ffi::KSTAT_TYPE_IO => KstatType::IO,
            ffi::KSTAT_TYPE_TIMER => KstatType::Timer,
            _ => panic!("invalid kstat type found"),
        }
    }
}
