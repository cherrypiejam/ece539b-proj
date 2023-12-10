use std::net::{IpAddr, Ipv4Addr};
use std::thread;
use std::sync::{Mutex, Arc};
use std::time::{Duration, Instant};

use clap::{Arg, Command};
use rand::RngCore;

mod icmp;
mod ping;
mod stream;
mod channel;
mod duplex;

fn main() {
    let matches = Command::new("net-timing-channel")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("transmit")
                .short_flag('t')
                .arg(Arg::new("interval"))
                .arg(Arg::new("duration"))
                .arg(Arg::new("data"))
        )
        .subcommand(
            Command::new("receive")
                .short_flag('r')
                .arg(Arg::new("interval"))
                .arg(Arg::new("duration"))
                .arg(Arg::new("buf-size"))
        )
        .subcommand(
            Command::new("benchmark")
                .arg(Arg::new("interval"))
                .arg(Arg::new("duration"))
        )
        .subcommand(
            Command::new("macrobenchmark")
                .arg(Arg::new("interval"))
                .arg(Arg::new("duration"))
        )
        .get_matches();


    match matches.subcommand() {
        Some(("transmit", tx_matches)) => {
            // let addr =
                // tx_matches
                // .get_one("addr")
                // .map(String::to_string)
                // .unwrap()
                // .parse::<Ipv4Addr>()
                // .unwrap();

            let interval = matches
                .get_one("interval")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No interval");

            let duration = matches
                .get_one("duration")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No duration");

            let config = channel::Config {
                interval,
                duration,
                burst_control: channel::BurstControl::TcpStream {
                    url: channel::options::NIXOS_IMG_URL.to_owned(),
                },
                baseline_hostname: channel::options::GOOGLE_HOSTNAME.to_owned(),
                baseline_threshold: Duration::from_millis(20),
                max_miss_rate_allowed: 0.15,
            };

            let data = tx_matches
                .get_one("data")
                .map(String::to_string)
                .expect("No data");

            let (sender, _) = channel::channel(config).expect("fail to create a channel");
            let _ = sender.send(data.as_bytes());

            println!("Sending done");
        }
        Some(("receive", rx_matches)) => {
            let interval = matches
                .get_one("interval")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No interval");

            let duration = matches
                .get_one("duration")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No duration");

            let config = channel::Config {
                interval,
                duration,
                burst_control: channel::BurstControl::TcpStream {
                    url: channel::options::NIXOS_IMG_URL.to_owned(),
                },
                baseline_hostname: channel::options::GOOGLE_HOSTNAME.to_owned(),
                baseline_threshold: Duration::from_millis(20),
                max_miss_rate_allowed: 0.15,
            };

            let buf_size = rx_matches
                .get_one("buf-size")
                .map(String::to_string)
                .and_then(|x| x.parse::<usize>().ok())
                .expect("No buffer size");

            let mut buf = vec![0u8; buf_size];

            let (_, receiver) = channel::channel(config).expect("fail to create a channel");
            let _ = receiver.recv(&mut buf);

            println!("Received: {}", String::from_utf8(buf).unwrap());
        }
        Some(("benchmark", matches)) => {
            let interval = matches
                .get_one("interval")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No interval");

            let duration = matches
                .get_one("duration")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No duration");

            let config = channel::Config {
                interval,
                duration,
                burst_control: channel::BurstControl::TcpStream {
                    url: channel::options::NIXOS_IMG_URL.to_owned(),
                },
                baseline_hostname: channel::options::GOOGLE_HOSTNAME.to_owned(),
                baseline_threshold: Duration::from_millis(20),
                max_miss_rate_allowed: 0.15,
            };

            let (sender, receiver) = channel::channel(config).expect("fail to create a channel");
            let mut handles = vec![];

            let size = 4;
            let buf = Arc::new(Mutex::new(vec![0u8; size]));
            let data = Arc::new(Mutex::new(vec![0u8; size]));

            let _ = data.lock().map(|mut d| {
                rand::thread_rng().fill_bytes(&mut d);
            });

            let data_dup = Arc::clone(&data);
            handles.push(thread::spawn(move || {
                let _ = data_dup.lock().map(|d| {
                    let _ = sender.send(&d);
                });
            }));

            let buf_dup = Arc::clone(&buf);
            handles.push(thread::spawn(move || {
                let _ = buf_dup.lock().map(|mut b| {
                    let _ = receiver.recv(&mut b);
                });
            }));

            handles.into_iter().for_each(|h| {
                let _ = h.join();
            });

            let _ = data.lock().map(|d| {
                let _ = buf.lock().map(|b| {
                    let errs = d.iter().zip(b.iter()).fold(0, |errs, (&a, &b)| {
                        let diff = a ^ b;
                        errs + diff.count_ones()
                    });
                    // println!(
                        // "data bits: {}, error bits: {}, error rate: {}",
                         // d.len() * 8,
                         // errs,
                         // errs as f32 / ((d.len() * 8) as f32),
                    // )
                    print!(
                        "{}",
                        errs as f32 / ((d.len() * 8) as f32),
                    )
                });
            });
        }
        Some(("macrobenchmark", matches)) => {
            let interval = matches
                .get_one("interval")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No interval");

            let duration = matches
                .get_one("duration")
                .map(String::to_string)
                .and_then(|x| x.parse::<u64>().ok())
                .map(Duration::from_millis)
                .expect("No duration");

            let config1 = channel::Config {
                interval,
                duration,
                burst_control: channel::BurstControl::TcpStream {
                    url: channel::options::NIXOS_IMG_URL.to_owned(),
                },
                baseline_hostname: channel::options::GOOGLE_HOSTNAME.to_owned(),
                baseline_threshold: Duration::from_millis(20),
                max_miss_rate_allowed: 0.15,
            };
            let config2 = config1.clone();

            let socket1 = duplex::socket(config1).expect("fail to create a channel");
            let socket2 = duplex::socket(config2).expect("fail to create a channel");

            let size = 1;
            let buf = Arc::new(Mutex::new(vec![0u8; size]));
            let data = Arc::new(Mutex::new(vec![0u8; size]));
            let mut handles = vec![];

            let _ = data.lock().map(|mut d| {
                rand::thread_rng().fill_bytes(&mut d);
            });

            let data_dup = Arc::clone(&data);
            handles.push(thread::spawn(move || {
                let _ = data_dup.lock().map(|d| {
                    let _ = socket1.send(&d);
                });
            }));

            let buf_dup = Arc::clone(&buf);
            handles.push(thread::spawn(move || {
                let _ = buf_dup.lock().map(|mut b| {
                    let _ = socket2.recv(&mut b);
                });
            }));

            handles.into_iter().for_each(|h| {
                let _ = h.join();
            });

            let _ = data.lock().map(|d| {
                let _ = buf.lock().map(|b| {
                    let errs = d.iter().zip(b.iter()).fold(0, |errs, (&a, &b)| {
                        let diff = a ^ b;
                        errs + diff.count_ones()
                    });
                    // println!(
                        // "data bits: {}, error bits: {}, error rate: {}",
                         // d.len() * 8,
                         // errs,
                         // errs as f32 / ((d.len() * 8) as f32),
                    // )
                    print!(
                        "{}",
                        errs as f32 / ((d.len() * 8) as f32),
                    )
                });
            });
        }
        _ => panic!("Bad args")
    }

}
