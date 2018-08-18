//#![deny(warnings)]
//#![deny(missing_docs)]

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
use std::io;
use std::marker::PhantomData;

mod ffi;
mod kstat_ctl;
/// The type of data found in named-value pairs of a kstat
pub mod kstat_named;

use kstat_ctl::{Kstat, KstatCtl, KstatData};

/// `KstatReader` represents all of the kstats that matched the fields of interest when created
/// with `KstatCtl.reader(...)`
#[derive(Debug)]
pub struct KstatReader<'a> {
    module: Option<Cow<'a, str>>,
    instance: Option<i32>,
    name: Option<Cow<'a, str>>,
    class: Option<Cow<'a, str>>,
    ctl: KstatCtl,
}

impl<'a> KstatReader<'a> {
    pub fn new<S>(
        module: Option<S>,
        instance: Option<i32>,
        name: Option<S>,
        class: Option<S>,
    ) -> io::Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        let ctl = KstatCtl::new()?;
        let module = module.map_or(None, |m| Some(m.into()));
        let name = name.map_or(None, |n| Some(n.into()));
        let class = class.map_or(None, |c| Some(c.into()));

        Ok(KstatReader {
            module,
            instance,
            name,
            class,
            ctl,
        })
    }

    pub fn read(&self) -> io::Result<Vec<KstatData>> {
        // First update the chain
        self.ctl.chain_update()?;

        let mut ret = Vec::new();
        let mut kstat_ptr = self.ctl.get_chain();
        while !kstat_ptr.is_null() {
            let kstat = Kstat {
                inner: kstat_ptr,
                _marker: PhantomData,
            };

            // Loop until we reach the end of the chain
            kstat_ptr = unsafe { (*kstat_ptr).ks_next };

            // must be NAMED or IO
            let ks_type = kstat.get_type();
            if ks_type != ffi::KSTAT_TYPE_NAMED && ks_type != ffi::KSTAT_TYPE_IO {
                continue;
            }

            // Compare against module/instance/name/class
            if self.module.is_some() && kstat.get_module() != *self.module.as_ref().unwrap() {
                continue;
            }

            if self.instance.is_some() && kstat.get_instance() != *self.instance.as_ref().unwrap() {
                continue;
            }

            if self.name.is_some() && kstat.get_name() != *self.name.as_ref().unwrap() {
                continue;
            }

            if self.class.is_some() && kstat.get_class() != *self.class.as_ref().unwrap() {
                continue;
            }

            ret.push(kstat.read(&self.ctl)?);
        }

        Ok(ret)
    }
}
