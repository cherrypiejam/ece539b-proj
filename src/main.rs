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

// fn recv_one(addr: IpAddr) -> Result<Duration> {
    // let ddl = Instant::now() + Duration::from_millis(1000);
    // let mut total: Vec<Result<Duration>> = vec![];
    // loop {
        // if Instant::now() > ddl {
            // break Ok(total.iter().fold(Duration::from_millis(0), |acc, x| {
                // if let Ok(x) = x.as_ref() {
                    // acc + *x
                // } else {
                    // acc + Duration::from_millis(0)
                // }
            // })/total.len() as u32);
        // }

        // let start = Instant::now();
        // total.push(
            // ping(addr.clone())
            // .map(|_| start.elapsed())
        // );
    // }
    // // let start = Instant::now();
    // // let _ = ping(addr)?;
    // // Ok(start.elapsed())
// }

// fn recv_buf(addr: IpAddr, buf: &mut [u8]) {
    // for byte in buf.iter_mut() {
        // let mut b = 0;
        // for i in 0..(std::mem::size_of::<u8>() * BITS_PER_BYTE) {
            // if recv_one(addr.clone()).unwrap() > THRESHOLD {
                // println!("recv data {}", 1);
                // b |= 1 << i
            // } else {
                // println!("recv data {}", 0);
            // }
        // }
        // *byte = b;
    // }
// }

// fn send_one(addr: Ipv4Addr, bit: bool) {
    // if bit {
        // println!("Send data 1");
        // let ddl = Instant::now() + Duration::from_secs(1000);
        // let handles =
            // (0..1)
            // .map(|_| {
                // let addr_dup = addr.clone();
                // let ddl_dup = ddl.clone();
                // thread::spawn(move || loop {
                    // if Instant::now() > ddl_dup {
                        // break
                    // } else {
                        // println!("ping one");
                        // // let _ = ping_noreply(addr_dup);
                        // stream::stream(addr_dup).unwrap();
                        // println!("ping one done");
                    // }
                // })
            // })
            // .collect::<Vec<_>>();
        // println!("{:?}", handles.len());
        // for handle in handles.into_iter() {
            // let _ = handle.join().expect("fail to join threads");
        // }
    // } else {
        // thread::sleep(Duration::from_secs(1000))
    // }
// }

// fn send_buf(addr: IpAddr, buf: &[u8]) {
    // for byte in buf.iter() {
        // for i in 0..(std::mem::size_of::<u8>() * BITS_PER_BYTE) {
            // println!("(byte >> i) & !1: {}", (byte >> i) & 1);
            // send_one(addr, (byte >> i) & 1 == 1);
        // }
    // }
// }

fn main() {
    let matches = Command::new("net-timing-channel")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("transmit")
                .short_flag('t')
                .arg(Arg::new("addr"))
                .arg(Arg::new("data"))
        )
        .subcommand(
            Command::new("receive")
                .short_flag('r')
                .arg(Arg::new("addr"))
        )
        .subcommand(
            Command::new("benchmark")
                .arg(Arg::new("interval"))
                .arg(Arg::new("duration"))
        )
        .get_matches();


    match matches.subcommand() {
        Some(("transmit", tx_matches)) => {
            let addr =
                tx_matches
                .get_one("addr")
                .map(String::to_string)
                .unwrap()
                .parse::<Ipv4Addr>()
                .unwrap();

            // let data = tx_matches
                // .get_one("data")
                // .map(String::to_string)
                // .expect("No data");

            // println!("{:?}", data);
            // send_buf(ip, data.as_bytes());
            // send_buf(ip, &[0b11111111, 0b11111111]);
            loop {
                // send_one(ip, true);
                // println!("begin");
                // let start = Instant::now();
                // let _ = ping::ping(addr);
                // println!("ping time {:?}", start.elapsed());
                println!("begin");
                // send_one(addr, true);
                println!("end...wait");
                thread::sleep(Duration::from_secs(1));
            }
        }
        Some(("receive", rx_matches)) => {
            let addr = IpAddr::V4(
                rx_matches
                .get_one("addr")
                .map(String::to_string)
                .unwrap()
                .parse::<Ipv4Addr>()
                .unwrap()
            );
            let buf = &mut [0u8, 100];
            // recv_buf(addr, buf);
            println!("received: {}", String::from_utf8(buf.to_vec()).unwrap());
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
        _ => panic!("Bad args")
    }

}
