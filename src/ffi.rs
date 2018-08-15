use byteorder::{ByteOrder, NativeEndian};
use libc::{c_char, c_int, c_longlong, c_uchar, c_uint, c_void, size_t};
use std::ffi::CStr;

pub const KSTAT_TYPE_RAW: c_uchar = 0; // can be anything
pub const KSTAT_TYPE_NAMED: c_uchar = 1; // name/value pair
pub const KSTAT_TYPE_INTR: c_uchar = 2; // interrupt statistics
pub const KSTAT_TYPE_IO: c_uchar = 3; // I/O statistics
pub const KSTAT_TYPE_TIMER: c_uchar = 4; // event timer
pub const KSTAT_NUM_TYPES: c_uchar = 5;

pub const KSTAT_STRLEN: usize = 31; // 30 chars + NULL; must be 16 * n - 1

pub const KSTAT_DATA_CHAR: c_uchar = 0;
pub const KSTAT_DATA_INT32: c_uchar = 1;
pub const KSTAT_DATA_UINT32: c_uchar = 2;
pub const KSTAT_DATA_INT64: c_uchar = 3;
pub const KSTAT_DATA_UINT64: c_uchar = 4;
pub const KSTAT_DATA_STRING: c_uchar = 9;

#[repr(C)]
#[derive(Debug)]
pub struct kstat_t {
    pub ks_crtime: c_longlong,             // creation time (from gethrtime())
    pub ks_next: *const kstat_t,           // kstat chain linkage
    pub ks_kid: c_int,                     // unique kstat ID
    pub ks_module: [c_char; KSTAT_STRLEN], // provicer module's name
    pub ks_resv: c_uchar,                  // reserved, currently just padding
    pub ks_instance: c_int,                // provider module's instance
    pub ks_name: [c_char; KSTAT_STRLEN],   // kstat name
    pub ks_type: c_uchar,                  // kstat data type
    pub ks_class: [c_char; KSTAT_STRLEN],  // kstat class
    pub ks_flags: c_uchar,                 // kstat flags
    pub ks_data: *const c_void,            // kstat type-specific data
    pub ks_ndata: c_uint,                  // # of type-specific data records
    pub ks_data_size: size_t,              // total size of kstat data section
    pub ks_snaptime: c_longlong,           // time of last data snapshot
    ks_update: extern "C" fn(kstat: *const kstat_t, c_int) -> c_int, // kernel only
    ks_private: *const c_void,             // kernel only
    ks_snapshot: extern "C" fn(kstat: *const kstat_t, c_int) -> c_int, // kernel only
    ks_lock: *const c_void,                // kernel only
}

impl kstat_t {
    pub fn get_class(&self) -> String {
        let cstr = unsafe { CStr::from_ptr(self.ks_class.as_ptr()) };
        cstr.to_string_lossy().into_owned()
    }
}

#[repr(C)]
pub struct kstat_ctl_t {
    pub kc_chain_id: c_int,       // current kstat chain ID
    pub kc_chain: *const kstat_t, // pointer to kstat chain
    pub kc_id: c_int,             // /dev/kstat descriptor
}

#[repr(C)]
pub struct kstat_named_t {
    pub name: [c_char; KSTAT_STRLEN], // name of counter
    pub data_type: c_uchar,           // data type
    pub value: [u8; 16],              // Union of fields
}

impl kstat_named_t {
    pub fn value_as_char(&self) -> c_char {
        c_char::from_le(self.value[0] as i8)
    }

    pub fn value_as_i32(&self) -> i32 {
        NativeEndian::read_i32(&self.value)
    }

    pub fn value_as_u32(&self) -> u32 {
        NativeEndian::read_u32(&self.value)
    }

    pub fn value_as_i64(&self) -> i64 {
        NativeEndian::read_i64(&self.value)
    }

    pub fn value_as_u64(&self) -> u64 {
        NativeEndian::read_u64(&self.value)
    }

    pub fn value_as_string(&self) -> String {
        let ptr = NativeEndian::read_u64(&self.value);
        let cstr = unsafe { CStr::from_ptr(ptr as *const c_char) };
        cstr.to_string_lossy().into_owned()
    }
}

#[link(name = "kstat")]
extern "C" {
    pub fn kstat_open() -> *const kstat_ctl_t;
    pub fn kstat_close(kc: *const kstat_ctl_t) -> c_int;
    pub fn kstat_chain_update(kc: *const kstat_ctl_t) -> c_int;
    pub fn kstat_lookup(
        kc: *const kstat_ctl_t,
        ks_module: *const c_char,
        ks_instance: c_int,
        ks_name: *const c_char,
    ) -> *const kstat_t;
    // Marking the buf as const instead of mut because we don't plan on using it in this API
    pub fn kstat_read(kc: *const kstat_ctl_t, ksp: *const kstat_t, buf: *const c_void) -> c_int;
}
