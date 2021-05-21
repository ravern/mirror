use std::{
  collections::HashMap, fs::read_to_string, io, path::PathBuf, thread,
};

use hotwatch::{
  blocking::{Flow, Hotwatch},
  Event,
};
use structopt::StructOpt;
use thiserror::Error;

use crate::net::{Client, Request};

mod net;
mod server;

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
  #[error("{0}")]
  Server(#[from] server::Error),
  #[error("failed to start watcher: {0}")]
  Hotwatch(#[from] hotwatch::Error),
}

pub fn run() -> Result<(), RunError> {
  let Config {
    sync_path,
    index_path: _,
    port,
    device_addrs,
  } = Config::from_args();

  let device_addrs_clone = device_addrs.clone();
  let sync_path_clone = sync_path.clone();

  thread::spawn(move || watch(sync_path_clone, device_addrs_clone).unwrap());

  Ok(server::listen(server::Config {
    sync_path,
    port,
    device_addrs,
  })?)
}

fn watch(
  sync_path: PathBuf,
  device_addrs: Vec<String>,
) -> Result<(), RunError> {
  let mut device_clients = HashMap::new();
  for addr in device_addrs {
    device_clients.insert(addr.clone(), Client::new(addr));
  }
  let mut hotwatch = Hotwatch::new()?;
  hotwatch.watch(sync_path.clone(), move |event| {
    match event {
      Event::Rename(from_path, to_path) => {
        println!("file renamed: from {:?}, to {:?}", from_path, to_path)
      }
      Event::Create(path) | Event::Write(path) => {
        println!("file created: {:?}", path);
        let relative_path =
          pathdiff::diff_paths(path.clone(), sync_path.clone())
            .expect("failed to diff paths");
        let contents = read_to_string(path).unwrap();
        for client in device_clients.values_mut() {
          let request = Request::put(relative_path.clone(), contents.clone());
          client.request(request).expect("request failed");
        }
      }
      Event::Remove(path) => {
        println!("file removed: {:?}", path);
        let relative_path = pathdiff::diff_paths(path, sync_path.clone())
          .expect("failed to diff paths");
        for client in device_clients.values_mut() {
          let request = Request::del(relative_path.clone());
          client.request(request).expect("request failed");
        }
      }
      _ => {}
    }
    Flow::Continue
  })?;
  hotwatch.run();

  Ok(())
}
