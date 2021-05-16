use std::{
  fs::read_to_string,
  io::{self, Read, Write},
  net::{TcpListener, TcpStream},
  path::PathBuf,
  thread,
};

use hotwatch::{
  blocking::{Flow, Hotwatch},
  Event,
};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Config {
  #[structopt(short, long, parse(from_os_str))]
  sync_path: PathBuf,
  #[structopt(short, long, parse(from_os_str))]
  index_path: PathBuf,
  #[structopt(short, long, default_value = "8999")]
  port: u16,
  #[structopt(short, long)]
  device_addrs: Vec<String>,
}

#[derive(Debug, Error)]
pub enum RunError {
  #[error("{0}")]
  Io(#[from] io::Error),
  #[error("failed to start watcher: {0}")]
  Hotwatch(#[from] hotwatch::Error),
}

pub fn run() -> Result<(), RunError> {
  let Config {
    sync_path,
    index_path,
    port,
    device_addrs,
  } = Config::from_args();

  let device_addrs_clone = device_addrs.clone();

  let listen_handle = thread::spawn(move || listen(port, device_addrs));
  let watch_handle =
    thread::spawn(move || watch(sync_path, device_addrs_clone));

  listen_handle.join().unwrap()?;
  watch_handle.join().unwrap()?;

  Ok(())
}

fn listen(port: u16, device_addrs: Vec<String>) -> Result<(), RunError> {
  let listener = TcpListener::bind("localhost:8999")?;
  loop {
    let (mut stream, addr) = listener.accept()?;
    let mut contents = "".to_string();
    stream.read_to_string(&mut contents)?;
    println!("read contents {}", contents);
  }
}

fn watch(
  sync_path: PathBuf,
  device_addrs: Vec<String>,
) -> Result<(), RunError> {
  let mut hotwatch = Hotwatch::new()?;
  hotwatch.watch(sync_path, move |event| {
    match event {
      Event::Rename(from_path, to_path) => {
        println!("file renamed: from {:?}, to {:?}", from_path, to_path)
      }
      Event::Create(path) => {
        println!("file created: {:?}", path);
        let contents = read_to_string(path).unwrap();
        println!("contents of file: {}", contents);
        for addr in device_addrs.iter() {
          let mut stream = TcpStream::connect(addr).unwrap();
          stream.write_all(contents.as_bytes()).unwrap();
          println!("file written to network");
        }
      }
      Event::Write(path) => println!("file written: {:?}", path),
      Event::Remove(path) => println!("file removed: {:?}", path),
      _ => {}
    }
    Flow::Continue
  })?;
  hotwatch.run();

  Ok(())
}
