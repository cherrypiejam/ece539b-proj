use anyhow::Result;

use crate::channel;


const MSG_HDR: u8 = 0x7c;
const MSG_ACK: u8 = 0xd3;
const MSG_SIZE: usize = 3;


// enum State {
    // InitListen,
    // InitConnect,
    // Send,
    // Receive,
    // None,
// }

pub struct Socket {
    sender: channel::Sender,
    receiver: channel::Receiver,
    // state: State,
}

pub fn socket(config: channel::Config) -> Result<Socket> {
    let (sender, receiver) = channel::channel(config)?;
    Ok(Socket{
        sender, receiver, // state: State::None,
    })
}

impl Socket {
    pub fn send(&self, buf: &[u8]) {
        let mut cur = 0;
        loop {
            if cur == buf.len() {
                break
            }

            let send_buf = [MSG_HDR, cur as u8, buf[cur]];
            dbg!("Send sth {:?}", send_buf);
            let _ = self.sender.send(&send_buf);

            for _ in 0..1 {
                println!("Send wait for reply");
                let mut recv_buf = [0u8; MSG_SIZE];
                self.receiver.recv(&mut recv_buf);
                if recv_buf[0] == MSG_ACK
                   && recv_buf[1] == cur as u8 {
                    cur += 1;
                    break
                }
            }
            dbg!("Send wait for reply failed, retring {:?}", cur);
        }
    }

    pub fn recv(&self, buf: &mut [u8]) {
        let mut flag = false;
        let mut retries = 0;
        loop {
            if flag && retries >= 5 {
                break
            }

            let mut recv_buf = [0u8; MSG_SIZE];
            dbg!("Receive try got");
            self.receiver.recv(&mut recv_buf);
            dbg!("Got something {:?}", recv_buf);

            if recv_buf[0] == MSG_HDR {
                dbg!("Got something checked {:?}", recv_buf);
                let index = recv_buf[1] as usize;
                buf[index] = recv_buf[2];
                let _ = self.sender.send(&[MSG_ACK, index as u8, 0x1]);
                retries = 0;
                flag = true;
            } else {
                retries += 1;
            }
        }
    }
}


