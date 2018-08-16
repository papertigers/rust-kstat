extern crate byteorder;
extern crate libc;

use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::ptr;

mod ffi;
pub mod kstat_named;
mod kstat_types;

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

    pub fn reader(
        &self,
        class: Option<&str>,
        module: Option<&str>,
        instance: Option<i32>,
    ) -> KstatReader {
        let mut kstats = Vec::new();
        let mut done = false;
        let mut kstat_ptr = unsafe { (*self.inner).kc_chain };

        while !done {
            let next = unsafe { (*kstat_ptr).ks_next };
            let kstat = Kstat {
                inner: kstat_ptr,
                _marker: PhantomData,
            };

            // Walk the chain until the end
            if next.is_null() {
                done = true;
            } else {
                kstat_ptr = next;
            }

            // must be NAMED or IO
            let ks_type = kstat.get_type();
            if ks_type != ffi::KSTAT_TYPE_NAMED && ks_type != ffi::KSTAT_TYPE_IO {
                continue;
            }

            // Compare against class/module/instance
            if class.is_some() && kstat.get_class() != *class.unwrap() {
                continue;
            }

            if module.is_some() && kstat.get_module() != *module.unwrap() {
                continue;
            }

            if instance.is_some() && kstat.get_instance() != instance.unwrap() {
                continue;
            }

            kstats.push(kstat);
        }

        KstatReader {
            inner: kstats,
            ctl: &self,
            _marker: PhantomData,
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
pub struct KstatData {
    pub class: String,
    pub module: String,
    pub instance: i32,
    pub name: String,
    pub snaptime: i64,
    pub data: HashMap<String, KstatNamedData>,
}

#[derive(Debug)]
pub struct Kstat<'ksctl> {
    inner: *const ffi::kstat_t,
    _marker: PhantomData<&'ksctl KstatCtl>,
}

impl<'ksctl> Kstat<'ksctl> {
    pub fn read(&self, ctl: &KstatCtl) -> io::Result<KstatData> {
        ctl.kstat_read(self)?;

        let class = self.get_class().into_owned();
        let module = self.get_module().into_owned();
        let instance = self.get_instance();
        let name = self.get_name().into_owned();
        let snaptime = self.get_snaptime();
        let data = self.get_data();
        Ok(KstatData {
            class,
            module,
            instance,
            name,
            snaptime,
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
    fn get_inner(&self) -> *const ffi::kstat_t {
        self.inner
    }

    #[inline]
    fn get_type(&self) -> libc::c_uchar {
        unsafe { (*self.get_inner()).ks_type }
    }

    #[inline]
    fn get_class(&self) -> Cow<str> {
        unsafe { (*self.inner).get_class() }
    }

    #[inline]
    fn get_module(&self) -> Cow<str> {
        unsafe { (*self.inner).get_module() }
    }

    #[inline]
    fn get_name(&self) -> Cow<str> {
        unsafe { (*self.inner).get_name() }
    }

    #[inline]
    fn get_instance(&self) -> i32 {
        unsafe { (*self.inner).ks_instance }
    }

    #[inline]
    fn get_snaptime(&self) -> i64 {
        unsafe { (*self.inner).ks_snaptime }
    }
}

#[derive(Debug)]
pub struct KstatReader<'a, 'ksctl> {
    inner: Vec<Kstat<'ksctl>>,
    ctl: &'a KstatCtl,
    _marker: PhantomData<&'ksctl KstatCtl>,
}

impl<'a, 'ksctl> KstatReader<'a, 'ksctl> {
    pub fn read(&self) -> io::Result<Vec<KstatData>> {
        // First update the chain
        self.ctl.chain_update()?;

        // Next loop the kstats of interest
        let mut ret = Vec::with_capacity(self.inner.len());
        for k in &self.inner {
            // TODO handle missing kstat by removing it from vec for future runs
            ret.push(k.read(self.ctl)?);
        }
        Ok(ret)
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
