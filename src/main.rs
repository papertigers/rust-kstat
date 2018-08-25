extern crate kstat;

use kstat::KstatReader;

fn main() {
    //let reader = KstatReader::new(None, None, None, Some("zone_vfs"))
    let reader =
        KstatReader::new::<String>(None, None, None, None).expect("failed to create kstat reader");
    let stats = reader.read().expect("failed to read kstats");
    println!("{:#?}", stats);
}
