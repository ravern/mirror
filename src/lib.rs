use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
  thread,
};

use structopt::StructOpt;
use thiserror::Error;

use crate::index::Index;

mod index;
mod net;
mod server;
mod watcher;

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
  Server(#[from] server::Error),
}

pub fn run() -> Result<(), RunError> {
  let Config {
    sync_path,
    index_path: _,
    port,
    device_addrs,
  } = Config::from_args();
  let index = Arc::new(Mutex::new(Index::new()));

  let device_addrs_clone = device_addrs.clone();
  let sync_path_clone = sync_path.clone();
  let index_clone = index.clone();

  thread::spawn(move || {
    watcher::watch(
      index_clone,
      watcher::Config {
        sync_path: sync_path_clone,
        device_addrs: device_addrs_clone,
      },
    )
    .unwrap()
  });

  Ok(server::listen(
    index,
    server::Config {
      sync_path,
      port,
      device_addrs,
    },
  )?)
}
