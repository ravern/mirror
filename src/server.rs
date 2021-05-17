use std::{
  fs, io,
  net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
  path::PathBuf,
  thread,
};

use thiserror::Error;

use crate::net::Request;

#[derive(Debug, Error)]
pub enum Error {
  #[error("{0}")]
  Io(#[from] io::Error),
}

#[derive(Clone, Debug)]
pub struct Config {
  pub sync_path: PathBuf,
  pub port: u16,
  pub device_addrs: Vec<String>,
}

pub fn handle(config: Config, mut stream: TcpStream) {
  match Request::parse(&mut stream).expect("parse request failed") {
    Request::Put { path, contents } => {
      println!("PUT {:?} {}", path, contents);
      let mut absolute_path = config.sync_path.clone();
      absolute_path.push(path);
      fs::write(absolute_path, contents).expect("failed to write file");
    }
    Request::Del { path } => {
      println!("DEL {:?}", path);
      let mut absolute_path = config.sync_path.clone();
      absolute_path.push(path);
      fs::remove_file(absolute_path).expect("failed to delete file");
    }
  }
}

pub fn listen(config: Config) -> Result<(), Error> {
  let listener = TcpListener::bind(format!("0.0.0.0:{}", &config.port))?;

  loop {
    let (stream, addr) = listener.accept()?;
    if is_device(&config.device_addrs, addr)? {
      let config_clone = config.clone();
      thread::spawn(move || handle(config_clone, stream));
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
