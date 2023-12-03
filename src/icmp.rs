use std::io::Write;

use anyhow::Result;

#[repr(C)]
pub struct ControlMsg {
    kind: u8,
    code: u8,
}

pub trait ICMP<'a> {
    const CNTL_MSG: ControlMsg;
    fn read(buf: &'a mut [u8]) -> Result<Self> where Self: Sized;
    fn write(&self, buf: &mut [u8]) -> Result<usize>;
}

// const ICMP_CNTL_ECHO: ControlMsg = ControlMsg { kind: 8, code: 0 };
// const ICMP_CNTL_ECHO_REPLY: ControlMsg = ControlMsg { kind: 0, code: 0 };

pub struct Echo<'a> {
    id: u16,
    seq: u16,
    data: &'a [u8],
}

impl<'a> Echo<'a> {
    pub fn new(id: u16, seq: u16, data: &'a [u8]) -> Self {
        Self { id, seq, data }
    }
}

impl<'a> ICMP<'a> for Echo<'a> {
    const CNTL_MSG: ControlMsg = ControlMsg { kind: 8, code: 0 };

    fn read(buf: &mut [u8]) -> Result<Self> {
        todo!()
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize> {
        buf[0] = Self::CNTL_MSG.kind;
        buf[1] = Self::CNTL_MSG.code;
        buf[2] = 0;
        buf[3] = 0;
        buf[4] = (self.id >> 8) as u8;
        buf[5] = self.id as u8;
        buf[6] = (self.seq >> 8) as u8;
        buf[7] = self.seq as u8;
        let writes = (&mut buf[8..]).write(self.data)?;
        let checksum = checksum(buf);
        buf[2] = (checksum >> 8) as u8;
        buf[3] = checksum as u8;
        Ok(writes + 8)
    }
}

pub struct EchoReply<'a> {
    id: u16,
    seq: u16,
    data: &'a [u8],
}

impl<'a> EchoReply<'a> {
    pub fn new(id: u16, seq: u16, data: &'a [u8]) -> Self {
        Self { id, seq, data }
    }
}

impl<'a> ICMP<'a> for EchoReply<'a> {
    const CNTL_MSG: ControlMsg = ControlMsg { kind: 0, code: 0 };

    fn read(buf: &'a mut [u8]) -> Result<Self> {
        if buf[0] != Self::CNTL_MSG.kind
        || buf[1] != Self::CNTL_MSG.code {
            anyhow::bail!("Bad echo reply")
        }

        Ok(EchoReply {
            id: (u16::from(buf[4]) << 8) + u16::from(buf[5]),
            seq: (u16::from(buf[6]) << 8) + u16::from(buf[7]),
            data: &buf[8..]
        })
    }

    fn write(&self, _buf: &mut [u8]) -> Result<usize> {
        todo!()
    }

}

fn checksum(buf: &[u8]) -> u16 {
    use std::io::Cursor;
    use byteorder::{BE, ReadBytesExt};

    let mut cur = Cursor::new(buf);
    let mut result = 0u16;
    while let Ok(value) = cur.read_u16::<BE>() {
        result = result.wrapping_add(value);
    }

    !result as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let data = &[0x3, 0x4, 0x5];
        let echo = Echo { id: 0x1, seq: 0x2, data };

        let buf = &mut [0; 32];
        let _ = echo.write(buf);
        println!("{:?}", buf);
    }
}
