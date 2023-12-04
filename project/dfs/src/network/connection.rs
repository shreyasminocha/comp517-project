use std::io::Error;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use std::net::TcpStream;

// todo: fix type safety stuff
pub fn recv_packet<T: Serialize + for<'a> Deserialize<'a>>(
    stream: &mut TcpStream,
) -> Result<T, Error> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let length = u32::from_ne_bytes(len_buf);
    let mut packet_buf = vec![0; length as usize];
    stream.read_exact(&mut packet_buf)?;
    Ok(serde_json::from_slice(&packet_buf)?)
}

pub fn send_packet<T: Serialize + for<'a> Deserialize<'a>>(
    stream: &mut TcpStream,
    packet: &T,
) -> Result<(), Error> {
    let buf = serde_json::to_vec(&packet)?;
    let length = buf.len() as u32;
    stream.write_all(&length.to_ne_bytes())?;
    stream.write_all(&buf)?;
    Ok(())
}
