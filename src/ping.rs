use std::net::{Ipv4Addr, IpAddr, SocketAddr};
use std::time::{Instant, Duration};
use std::io::Read;
use std::thread;

use socket2::{Domain, Protocol, Socket, Type, SockAddr};
use anyhow::Result;
use rand::{self, RngCore};

use crate::icmp::{Echo, EchoReply, ICMP};

fn _ping(
    addr: Ipv4Addr,
    socket_type: Type,
    wait_for_reply: bool,
    write_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
) -> Result<()> {
    let addr = SockAddr::from(SocketAddr::new(IpAddr::V4(addr), 0));

    let buf = &mut [0u8; 64];
    let data = &mut [0u8; 64 - 8];
    rand::thread_rng().fill_bytes(data);

    let echo = Echo::new(rand::random(), 0, data);
    echo.write(buf)?;

    let mut sk = Socket::new(Domain::IPV4, socket_type, Some(Protocol::ICMPV4))?;

    sk.set_ttl(64)?;
    sk.set_write_timeout(write_timeout)?;
    sk.set_read_timeout(read_timeout)?;

    sk.send_to(buf, &addr)?;

    if wait_for_reply {
        let recv_buf = &mut [0u8; 1024];
        sk.read(recv_buf)?;
    }

    Ok(())
}

pub fn ping(addr: Ipv4Addr, timeout: Option<Duration>) -> Result<()> {
    _ping(addr, Type::DGRAM, true, timeout, timeout)
}

pub fn ping_noreply(addr: Ipv4Addr) -> Result<()> {
    let timeout = Some(Duration::from_millis(500));
    _ping(addr, Type::DGRAM, false, timeout, timeout)
}

pub fn ping_raw(addr: Ipv4Addr, timeout: Option<Duration>) -> Result<()> {
    _ping(addr, Type::RAW, true, timeout, timeout)
}

pub fn flood(addr: Ipv4Addr, duration: Duration, concurrency: usize) -> Result<()> {
    let ddl = Instant::now() + duration;
    let handles =
        (0..concurrency)
        .map(|_| {
            thread::spawn(move || loop {
                (0..1_000_000)
                    .for_each(|_| {
                        let _ = ping_noreply(addr);
                    });
                if Instant::now() > ddl {
                    break
                }
            })
        })
        .collect::<Vec<_>>();
    for handle in handles.into_iter() {
        let _ = handle.join().expect("fail to join threads");
    }
    Ok(())
}
