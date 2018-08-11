extern crate kstat;

use kstat::KstatCtl;

fn main() {
    // create handle to /dev/kstat
    let ctl = KstatCtl::new().expect("failed to open /dev/kstat");

    // lookup a kstat
    let caps = ctl.lookup(None, 31, Some("cpucaps_zone_31"))
        .expect("failed to lookudatap kstat");

    // get the kstat's data back as a hashmap
    let hash = caps.to_hashmap(&ctl).expect("failed to read kstat");
    for (key, value) in &hash {
        println!("{} - {:?}", key, value);
    }
}
