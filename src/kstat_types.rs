use super::ffi;
use libc::c_uchar;

#[derive(Debug)]
pub enum KstatType {
    Raw,
    Named,
    Interrupt,
    IO,
    Timer,
    Num,
}

impl From<c_uchar> for KstatType {
    fn from(t: c_uchar) -> Self {
        match t {
            ffi::KSTAT_TYPE_RAW => KstatType::Raw,
            ffi::KSTAT_TYPE_NAMED => KstatType::Named,
            ffi::KSTAT_TYPE_INTR => KstatType::Interrupt,
            ffi::KSTAT_TYPE_IO => KstatType::IO,
            ffi::KSTAT_TYPE_TIMER => KstatType::Timer,
            ffi::KSTAT_NUM_TYPES => KstatType::Num,
            _ => panic!("invalid kstat type found"),
        }
    }
}
