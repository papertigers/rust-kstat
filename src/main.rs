extern crate kstat;

use kstat::KstatCtl;

fn main() {
    let ctl = KstatCtl::new().expect("failed to open /dev/kstat");
    let reader = ctl.reader(None, None, None, Some("zone_vfs"));
    let stats = reader.read().expect("failed to read kstats");
    for stat in stats {
        println!("{:#?}", stat);
    }
}
