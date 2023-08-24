

mod scan;
use crate::scan::*;

#[async_std::main]
async fn main() {
    match scan_once_for_devices().await {
        Ok(addrs) => println!("Found cast devices: {:?}", addrs),
        Err(e) => eprintln!("error polling: {:?}", e),
    }
}
