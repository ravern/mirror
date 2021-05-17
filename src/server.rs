use std::{
  io::{self, Read},
  net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
  thread,
};

use thiserror::Error;

use crate::net::Request;

#[derive(Debug, Error)]
pub enum Error {
  #[error("{0}")]
  Io(#[from] io::Error),
}

#[derive(Debug)]
pub struct Config {
  pub port: u16,
  pub device_addrs: Vec<String>,
}

pub fn handle(mut stream: TcpStream) {
  match Request::parse(&mut stream).expect("parse request failed") {
    Request::Put { path, contents } => {
      println!("PUT {:?} {}", path, contents);
    }
    Request::Del { path } => {
      println!("DEL {:?}", path);
    }
  }
}

pub fn listen(config: Config) -> Result<(), Error> {
  let Config { port, device_addrs } = config;

  let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

  loop {
    let (stream, addr) = listener.accept()?;
    if is_device(&device_addrs, addr)? {
      thread::spawn(move || handle(stream));
    } else {
      println!("not in device list. blocked!")
    }
  }
}

fn is_device(
  device_addrs: &[String],
  incoming_addr: SocketAddr,
) -> io::Result<bool> {
  for addr in device_addrs {
    let socket_addrs = addr.to_socket_addrs()?;
    for socket_addr in socket_addrs {
      if socket_addr.ip() == incoming_addr.ip() {
        return Ok(true);
      }
    }
  }
  Ok(false)
}
