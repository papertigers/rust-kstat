# kstat

This rust library provides an ffi wrapper around the native illumos library.

### Example

The following is equivalent to `kstat -p -c zone_caps`:

```rust
extern crate kstat;
use kstat::KstatCtl;
fn main() {
    let ctl = KstatCtl::new().expect("failed to open /dev/kstat");
    let reader = ctl.reader(Some("zone_caps"), None, None);
    let stats = reader.read().expect("failed to read kstats");
    for stat in stats {
        println!("{:#?}", stat);
    }
}
```
