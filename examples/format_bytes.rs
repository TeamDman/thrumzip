use humansize::{DECIMAL, format_size};
use uom::si::f32::*;
use uom::si::information::{byte, gigabyte, kilobyte, megabyte};

fn main() {
    // Let's say we have 5_368_709_120 bytes (about 5 GB)
    let bytes = Information::new::<byte>(5_368_709_120.0);

    // Convert to kilobytes, megabytes, gigabytes
    let kb = bytes.get::<kilobyte>();
    let mb = bytes.get::<megabyte>();
    let gb = bytes.get::<gigabyte>();

    println!("{:.2} bytes", bytes.get::<byte>());
    println!("{:.2} KB", kb);
    println!("{:.2} MB", mb);
    println!("{:.2} GB", gb);
    println!("{} (auto)", format_size(bytes.get::<byte>() as u64, DECIMAL));
}
