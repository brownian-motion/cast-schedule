use std::fmt::Debug;
use regex::Regex;
use std::hash::Hash;
use futures_core::Stream;
use futures_util::stream::StreamExt;
use futures_util::pin_mut;
use mdns::RecordKind;
use mdns::*;
use std::collections::hash_set::*;
use std::net::*;
use std::time::Duration;
    use once_cell::sync::Lazy;

const CAST_PORT: u16 = 8009;
const SERVICE_NAME: &'static str = "_googlecast._tcp.local";
const POLL_FREQUENCY: Duration = Duration::from_secs(2);

pub type MdnsResult<T> = Result<T, mdns::Error>;

pub type SinglePollResult = MdnsResult<FoundDevice>;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct  FoundDevice {
    pub addr: SocketAddr,
    pub name: String,
    pub hostname: String,
}

pub async fn scan_once_for_devices() -> MdnsResult<Vec<FoundDevice>> {
    let stream = scan_for_devices()?;
    take_until_duplicate(stream).await
}

pub fn scan_for_devices() -> MdnsResult<impl Stream<Item = MdnsResult<FoundDevice>>> {
    // Iterate through responses from each Cast device, asking for new devices every 2s
    let stream = mdns::discover::all(SERVICE_NAME, POLL_FREQUENCY)?.listen();

    Ok(stream.filter_map(get_found_device))
}

async fn get_found_device(response: MdnsResult<mdns::Response>) -> Option<MdnsResult<FoundDevice>> {
    let r = match response {
        Ok(r) => r,
        Err(e) => return Some(Err(e)),
    };
    let addr = r.ip_addr().map(|ip_addr| SocketAddr::new(ip_addr, CAST_PORT));
    let hostname = r.hostname().map(|s| s.to_string());
    let name = find_friendly_name(r.txt_records()).unwrap_or("UNNAMED".to_string());
   addr.zip(hostname).map(|(addr, hostname)| Ok(FoundDevice{ addr: addr, hostname: hostname, name: name }))
}

fn find_friendly_name<'a, I : Iterator<Item = &'a str>>(txt_records: I) -> Option<String> {
    static FN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\bfn=([^;\"]+)").unwrap());

    txt_records
        .filter_map(|s| FN_REGEX.captures(s))
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .next()
}

async fn take_until_duplicate<T: Hash + Eq, S: Stream<Item=MdnsResult<T>>>(stream: S) -> MdnsResult<Vec<T>> {
    let mut seen = HashSet::new();

    pin_mut!(stream);
    while let Some(elem) = stream.next().await {
        let elem = elem?;
        let found_duplicate = seen.contains(&elem);
        seen.insert(elem);
        
        if found_duplicate {
            break;
        }
    }

    Ok(seen.into_iter().collect::<Vec<_>>())
}
