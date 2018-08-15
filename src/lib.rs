extern crate byteorder;
extern crate libc;

use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::ptr;

mod ffi;
mod kstat_lookup;
pub mod kstat_named;
mod kstat_types;

use kstat_lookup::KstatTriplet;
use kstat_named::{KstatNamed, KstatNamedData};

#[derive(Debug)]
pub struct KstatCtl {
    inner: *const ffi::kstat_ctl_t,
}

impl KstatCtl {
    /// Creates a new Kstat and initializes the underlying connection with `/dev/kstat`.
    pub fn new() -> io::Result<Self> {
        unsafe { ptr_or_err(ffi::kstat_open()).map(|c| KstatCtl { inner: c }) }
    }

    /// Updates the kstat chain, returning true if the chain has changed.
    pub fn chain_update(&self) -> io::Result<bool> {
        let ret = unsafe { chain_updated(ret_or_err(ffi::kstat_chain_update(self.inner))?) };
        Ok(ret)
    }

    /// Traverses the kstat chain searching for a kstat with the same module, instance, and name
    /// fields. The module and name fields can be ignored in a search by passing in `None`. The
    /// instance can also be ignored by passing in `-1`.
    pub fn lookup(
        &self,
        module: Option<&str>,
        instance: i32,
        name: Option<&str>,
    ) -> io::Result<Kstat> {
        let c_module = KstatTriplet::new(module)?;
        let c_name = KstatTriplet::new(name)?;

        unsafe {
            ptr_or_err(ffi::kstat_lookup(
                self.inner,
                c_module.as_ptr(),
                instance,
                c_name.as_ptr(),
            )).map(|k| Kstat {
                inner: k,
                _marker: PhantomData,
            })
        }
    }

    /// Gets the relevant data from the kernel for the kstat pointed to by the kstat argument.
    fn kstat_read(&self, kstat: &Kstat) -> io::Result<i32> {
        unsafe { ret_or_err(ffi::kstat_read(self.inner, kstat.get_inner(), ptr::null())) }
    }
}

impl Drop for KstatCtl {
    fn drop(&mut self) {
        let _ = unsafe { ffi::kstat_close(self.inner) };
    }
}

#[derive(Debug)]
pub struct Kstat<'ksctl> {
    inner: *const ffi::kstat_t,
    _marker: PhantomData<&'ksctl KstatCtl>,
}

impl<'ksctl> Kstat<'ksctl> {
    /// Returns a `HashMap` for the given kstat and all of its data fields
    pub fn to_hashmap(&self, ctl: &KstatCtl) -> io::Result<HashMap<String, KstatNamedData>> {
        ctl.kstat_read(self)?;
        unsafe {
            let ndata = (*self.inner).ks_ndata;
            let head = (*self.inner).ks_data as *const ffi::kstat_named_t;
            let mut ret = HashMap::with_capacity(ndata as usize + 3);
            ret.insert(
                String::from("class"),
                KstatNamedData::DataString((*self.inner).get_class()),
            );
            ret.insert(
                String::from("snaptime"),
                KstatNamedData::DataInt64((*self.inner).ks_snaptime),
            );
            ret.insert(
                String::from("crtime"),
                KstatNamedData::DataInt64((*self.inner).ks_crtime),
            );

            for i in 0..ndata {
                let (key, value) = KstatNamed::new(head.offset(i as isize)).read();
                ret.insert(key, value);
            }

            Ok(ret)
        }
    }

    fn get_inner(&self) -> *const ffi::kstat_t {
        self.inner
    }
}

// ============ Helpers ============

fn ptr_or_err<T>(ptr: *const T) -> io::Result<*const T> {
    if ptr.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(ptr)
    }
}

fn ret_or_err(ret: i32) -> io::Result<i32> {
    match ret {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(ret),
    }
}

fn chain_updated(kid: i32) -> bool {
    match kid {
        0 => false,
        _ => true,
    }
}
