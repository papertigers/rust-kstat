# kstat

[Docs](https://us-east.manta.joyent.com/mikezeller/public/rust/kstat/index.html)

This rust library provides an ffi wrapper around the native illumos library.

### Example

The following is equivalent to `kstat -p -n zone_vfs`:

```rust
extern crate kstat;

use kstat::KstatReader;

fn main() {
    let reader = KstatReader::new(None, None, None, Some("zone_vfs"))
        .expect("failed to create kstat reader");
    let stats = reader.read().expect("failed to read kstats");
    println!("{:#?}", stats);
}
```


