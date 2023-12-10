use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr};
use std::thread;

use dns_lookup;
use anyhow::Result;

use crate::ping;
use crate::stream;

const BITS_PER_BYTE: usize = 8;

#[derive(Clone)]
pub enum BurstControl {
    Ping {
        hostname: String,
        concurrency: usize,
    },
    TcpStream {
        url: String,
    },
}


fn burst(control: &BurstControl, duration: Duration) -> Result<()> {
    match control {
        BurstControl::Ping {
            hostname, concurrency
        } => {
            let ip = dns_lookup::lookup_host(&hostname).unwrap()[0];
            if let IpAddr::V4(ip) = ip {
                let _ = ping::flood(ip, duration, *concurrency); // ping flood
                Ok(())
            } else {
                anyhow::bail!("ICMPv6 not supported")
            }
        }
        BurstControl::TcpStream {
            url
        } => {
            stream::burst(&url, duration)
        }
    }
}

pub struct Sender {
    interval: Duration,
    duration: Duration,
    burst_control: BurstControl,
}

pub struct Receiver {
    interval: Duration,
    duration: Duration,
    baseline_hostname: String,
    baseline_threshold: Duration,
    max_miss_rate_allowed: f32,
}

#[derive(Clone)]
pub struct Config {
    pub interval: Duration,
    pub duration: Duration,
    pub burst_control: BurstControl,
    pub baseline_hostname: String,
    pub baseline_threshold: Duration,
    pub max_miss_rate_allowed: f32,
}

impl Default for Config {
    fn default() -> Self {
        let interval = Duration::from_secs(1);
        let duration = Duration::from_secs(1);
        let burst_control = BurstControl::TcpStream {
            url: options::NIXOS_IMG_URL.to_owned(),
        };
        let baseline_hostname = options::GOOGLE_HOSTNAME.to_owned();
        let baseline_threshold = Duration::from_millis(50);
        let max_miss_rate_allowed = 0.0;
        Config {
            interval,
            duration,
            burst_control,
            baseline_hostname,
            baseline_threshold,
            max_miss_rate_allowed,
        }
    }
}

pub fn channel(config: Config) -> Result<(Sender, Receiver)> {
    Ok((
        Sender {
            interval: config.interval,
            duration: config.duration,
            burst_control: config.burst_control,
        },
        Receiver {
            interval: config.interval,
            duration: config.duration,
            baseline_hostname: config.baseline_hostname,
            baseline_threshold: config.baseline_threshold,
            max_miss_rate_allowed: config.max_miss_rate_allowed,
        },
    ))
}

impl Sender {
    fn send_bit(&self, bit: bool) -> Result<()> {
        if bit {
            let ret = burst(&self.burst_control, self.duration);
            thread::sleep(self.interval);
            ret
        } else {
            Ok(thread::sleep(self.duration + self.interval))
        }
    }

    pub fn send(&self, buf: &[u8]) -> Result<()> {
        for byte in buf.iter() {
            for i in 0..(std::mem::size_of::<u8>() * BITS_PER_BYTE) {
                // println!("Sender: sending {}", (byte >> i) & 1);
                let _ = self.send_bit((byte >> i) & 1 == 1);
            }
        }
        Ok(())
    }
}



impl Receiver {
    fn recv_bit(&self, ip: Ipv4Addr) -> bool {
        let ddl = Instant::now() + self.duration;
        let mut measures = 0;
        let mut misses = 0;

        loop {
            if Instant::now() + self.baseline_threshold > ddl {
                break
            }

            // blocking
            if let Err(_) = ping::ping(ip, Some(self.baseline_threshold)) {
                misses += 1;
            }

            measures += 1;
        }

        let miss_rate = misses as f32 / measures as f32;

        // println!("Receiver: measured {:?}, ratio {:?}", measures, miss_rate);

        thread::sleep(self.interval);

        miss_rate > self.max_miss_rate_allowed
    }

    pub fn recv(&self, buf: &mut [u8]) {
        if let IpAddr::V4(ip) =
            dns_lookup::lookup_host(&self.baseline_hostname).expect("Bad hostname")[0]
        {
            for byte in buf.iter_mut() {
                *byte = 0u8;
                for i in 0..(std::mem::size_of::<u8>() * BITS_PER_BYTE) {
                    if self.recv_bit(ip) {
                        *byte |= 1 << i;
                    }
                }
            }
        } else {
            panic!("Ipv6 not supported");
        }
    }
}


pub mod options {
    pub const GOOGLE_HOSTNAME: &str = "gongqihuang.com";
    pub const NIXOS_IMG_URL: &str =
        "https://channels.nixos.org/nixos-23.11/latest-nixos-minimal-x86_64-linux.iso";
}
