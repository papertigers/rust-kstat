extern crate kstat;

use kstat::KstatReader;

fn main() {
    let mut reader = KstatReader::new().expect("failed to create kstat reader");
    reader.module("zone_vfs").class("zone_vfs");
    let stats = reader.read().expect("failed to read kstats");
    println!("{:#?}", stats);
}
