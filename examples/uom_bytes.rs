use humansize::DECIMAL;
use humansize::format_size;
use std::time::Duration;
use uom::fmt::DisplayStyle::Abbreviation;
use uom::si::f32::InformationRate;
use uom::si::f32::*;
use uom::si::information::byte;
use uom::si::information::gigabyte;
use uom::si::information::kilobyte;
use uom::si::information::megabyte;
use uom::si::information_rate::byte_per_second;
use uom::si::information_rate::kilobyte_per_second;
use uom::si::time::second;

fn main() {
    // Let's say we have 5_368_709_120 bytes (about 5 GB)
    let bytes = Information::new::<byte>(5_368_709_120.0);

    // Convert to kilobytes, megabytes, gigabytes
    let kb = bytes.get::<kilobyte>();
    let mb = bytes.get::<megabyte>();
    let gb = bytes.get::<gigabyte>();

    println!("{:.2} bytes", bytes.get::<byte>());
    println!("{kb:.2} KB");
    println!("{mb:.2} MB");
    println!("{gb:.2} GB");
    println!(
        "{} (auto)",
        format_size(bytes.get::<byte>() as u64, DECIMAL)
    );

    let bps = InformationRate::new::<kilobyte_per_second>(1.0);
    println!(
        "bps = {}",
        (bps).into_format_args(kilobyte_per_second, Abbreviation)
    );
    println!("bps = {bps:#?}");
    println!("bps = {bps:?}");

    let bytes = Information::new::<byte>(2.0);
    let duration = Time::new::<second>(10.0);
    let rate: InformationRate = (bytes / duration).into(); // .into() needed to convert betwen Kind and InformationKind
    println!("A: {:#?}", duration / bytes);
    println!("B: {:#?}", bytes / duration);
    println!(
        "C: {}",
        rate.into_format_args(byte_per_second, Abbreviation)
    );

    println!(
        "D: {} / {} = {}",
        humansize::format_size(bytes.get::<byte>() as u64, DECIMAL),
        humantime::format_duration(Duration::from_secs(duration.get::<second>() as u64)),
        (rate).into_format_args(byte_per_second, Abbreviation)
    );
}
