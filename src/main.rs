#![feature(async_fn_in_trait)]
#![feature(const_option)]

mod scan;
use scan::*;

mod model;
use model::*;

mod calendar;

mod draw;

#[async_std::main]
async fn main() {
    let _devices = match scan_once_for_devices().await {
        Ok(devices) => { report_devices(&devices); devices }
        Err(e) => { eprintln!("error polling: {:?}", e); Vec::new() },
    };
}

fn report_devices(devices: &[FoundDevice]) {
    println!("Found devices:");
    let width = devices.iter().map(|d| d.name.len()).max().unwrap_or(0);
    for device in devices {
        println!("\t{:width$}\t{}", device.name, device.addr, width = width)
    }
}
