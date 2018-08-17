#![deny(warnings)]
#![deny(missing_docs)]

//! # kstat
//!
//! A simple rust crate that allows you to read kernel statistics via the kstat framework on
//! illumos. The `kstat` crate exposes a `KstatReader` type that tracks kstats that are of
//! interest to the consumer, allowing them to call the `read` method on the type to read in all of
//! the named-value pairs associated with those particular kstats. This means that the crate only
//! allows the consumer to track/read kstats that are of type KSTAT_TYPE_NAMED or KSTAT_TYPE_IO.
//!
//! # Example:
//! ```
//! extern crate kstat;
//!
//! use kstat::KstatCtl;
//!
//! fn main() {
//!     // Open a kstat_ctl_t handle
//!     let ctl = KstatCtl::new().expect("failed to open /dev/kstat");
//!
//!     // Create a KstatReader that tracks kstat(s) in the "zone_caps" class
//!     let reader = ctl.reader(None, None, None, Some("zone_caps"));
//!
//!     // Call read on the  KstatReader to read in kstat(s) and their fields
//!     let stats = reader.read().expect("failed to read kstats");
//!
//!     // Loop over all of the returned `KstatData`s and debug print them
//!     for stat in stats {
//!         println!("{:#?}", stat);
//!     }
//! }
//!

extern crate byteorder;
extern crate libc;

use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::ptr;

mod ffi;
/// The type of data found in named-value pairs of a kstat
pub mod kstat_named;
mod kstat_types;

use kstat_named::{KstatNamed, KstatNamedData};

/// A wrapper around a `kstat_ctl_t` handle.
#[derive(Debug)]
pub struct KstatCtl {
    inner: *const ffi::kstat_ctl_t,
}

impl KstatCtl {
    /// Creates a new Kstat and initializes the underlying connection with `/dev/kstat`.
    ///
    /// # Example
    /// ```
    /// let ctl = kstat::KstatCtl::new().expect("failed to open /dev/kstat");
    /// ```
    pub fn new() -> io::Result<Self> {
        unsafe { ptr_or_err(ffi::kstat_open()).map(|c| KstatCtl { inner: c }) }
    }

    /// Updates the kstat chain, returning true if the chain has changed.
    fn chain_update(&self) -> io::Result<bool> {
        let ret = unsafe { chain_updated(ret_or_err(ffi::kstat_chain_update(self.inner))?) };
        Ok(ret)
    }

    /// Returns a `KstatReader` that tracks the kstats of interest.
    ///
    /// * `module` - optional string denoting module of kstat(s) to read
    /// * `instance` - optional int denoting instance of kstat(s) to read
    /// * `name` - optional string denoting name of kstat(s) to read
    /// * `class` - optional string denoting class of kstat(s) to read
    ///
    /// # Example
    /// ```
    /// # let ctl = kstat::KstatCtl::new().expect("failed to open /dev/kstat");
    /// let reader = ctl.reader(None, None, None, Some("zone_caps"));
    /// ```
    pub fn reader(
        &self,
        module: Option<&str>,
        instance: Option<i32>,
        name: Option<&str>,
        class: Option<&str>,
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

            // Compare against module/instance/name/class
            if module.is_some() && kstat.get_module() != *module.unwrap() {
                continue;
            }

            if instance.is_some() && kstat.get_instance() != instance.unwrap() {
                continue;
            }

            if name.is_some() && kstat.get_name() != *name.unwrap() {
                continue;
            }

            if class.is_some() && kstat.get_class() != *class.unwrap() {
                continue;
            }

            kstats.push(kstat);
        }

        KstatReader {
            inner: kstats,
            ctl: &self,
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

/// The corresponding data read in from a kstat
#[derive(Debug)]
pub struct KstatData {
    /// string denoting class of kstat
    pub class: String,
    /// string denoting module of kstat
    pub module: String,
    /// int denoting instance of kstat
    pub instance: i32,
    /// string denoting name of kstat
    pub name: String,
    /// nanoseconds since boot of this snapshot
    pub snaptime: i64,
    /// creation time of this kstat in nanoseconds since boot
    pub crtime: i64,
    /// A hashmap of the named-value pairs for the kstat
    pub data: HashMap<String, KstatNamedData>,
}

/// Wrapper around a kstat pointer
#[derive(Debug)]
struct Kstat<'ksctl> {
    inner: *const ffi::kstat_t,
    _marker: PhantomData<&'ksctl KstatCtl>,
}

impl<'ksctl> Kstat<'ksctl> {
    /// Read this particular kstat and its corresponding data into a `KstatData`
    fn read(&self, ctl: &KstatCtl) -> io::Result<KstatData> {
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

    #[inline]
    fn get_crtime(&self) -> i64 {
        unsafe { (*self.inner).ks_crtime }
    }
}

/// `KstatReader` represents all of the kstats that matched the fields of interest when created
/// with `KstatCtl.reader(...)`
#[derive(Debug)]
pub struct KstatReader<'a> {
    inner: Vec<Kstat<'a>>,
    ctl: &'a KstatCtl,
}

impl<'a> KstatReader<'a> {
    /// Calling read on the Reader will update the kstat chain and proceed to read each kstat and
    /// its corresponding data.
    ///
    /// # Example
    /// ```
    /// # let ctl = kstat::KstatCtl::new().expect("failed to open /dev/kstat");
    /// # let reader = ctl.reader(None, None, None, Some("zone_caps"));
    /// let stats = reader.read().expect("failed to read kstat");
    /// ```
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
