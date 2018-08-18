use super::ffi;
use super::kstat_named::{KstatNamed, KstatNamedData};
use KstatData;

use libc;

use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::ptr;

/// A wrapper around a `kstat_ctl_t` handle.
#[derive(Debug)]
pub struct KstatCtl {
    inner: *const ffi::kstat_ctl_t,
}

impl KstatCtl {
    pub fn new() -> io::Result<Self> {
        unsafe { ptr_or_err(ffi::kstat_open()).map(|c| KstatCtl { inner: c }) }
    }

    pub fn get_chain(&self) -> *const ffi::kstat_t {
        unsafe { (*self.inner).kc_chain }
    }

    pub fn chain_update(&self) -> io::Result<bool> {
        let ret = unsafe { chain_updated(ret_or_err(ffi::kstat_chain_update(self.inner))?) };
        Ok(ret)
    }

    pub fn kstat_read(&self, kstat: &Kstat) -> io::Result<i32> {
        unsafe { ret_or_err(ffi::kstat_read(self.inner, kstat.get_inner(), ptr::null())) }
    }
}

impl Drop for KstatCtl {
    fn drop(&mut self) {
        let _ = unsafe { ffi::kstat_close(self.inner) };
    }
}

/// Wrapper around a kstat pointer
#[derive(Debug)]
pub struct Kstat<'ksctl> {
    pub inner: *const ffi::kstat_t,
    pub _marker: PhantomData<&'ksctl KstatCtl>,
}

impl<'ksctl> Kstat<'ksctl> {
    /// Read this particular kstat and its corresponding data into a `KstatData`
    pub fn read(&self, ctl: &KstatCtl) -> io::Result<KstatData> {
        ctl.kstat_read(self)?;

        let class = self.get_class().into_owned();
        let module = self.get_module().into_owned();
        let instance = self.get_instance();
        let name = self.get_name().into_owned();
        let snaptime = self.get_snaptime();
        let crtime = self.get_crtime();
        let data = self.get_data();
        Ok(KstatData {
            class,
            module,
            instance,
            name,
            snaptime,
            crtime,
            data,
        })
    }

    fn get_data(&self) -> HashMap<String, KstatNamedData> {
        let head = unsafe { (*self.inner).ks_data as *const ffi::kstat_named_t };
        let ndata = unsafe { (*self.inner).ks_ndata };
        let mut ret = HashMap::with_capacity(ndata as usize);
        for i in 0..ndata {
            let (key, value) = KstatNamed::new(unsafe { head.offset(i as isize) }).read();
            ret.insert(key, value);
        }

        ret
    }

    #[inline]
    pub fn get_inner(&self) -> *const ffi::kstat_t {
        self.inner
    }

    #[inline]
    pub fn get_type(&self) -> libc::c_uchar {
        unsafe { (*self.get_inner()).ks_type }
    }

    #[inline]
    pub fn get_class(&self) -> Cow<str> {
        unsafe { (*self.inner).get_class() }
    }

    #[inline]
    pub fn get_module(&self) -> Cow<str> {
        unsafe { (*self.inner).get_module() }
    }

    #[inline]
    pub fn get_name(&self) -> Cow<str> {
        unsafe { (*self.inner).get_name() }
    }

    #[inline]
    pub fn get_instance(&self) -> i32 {
        unsafe { (*self.inner).ks_instance }
    }

    #[inline]
    pub fn get_snaptime(&self) -> i64 {
        unsafe { (*self.inner).ks_snaptime }
    }

    #[inline]
    pub fn get_crtime(&self) -> i64 {
        unsafe { (*self.inner).ks_crtime }
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
