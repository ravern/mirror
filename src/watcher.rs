use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use hotwatch::{
  blocking::{Flow, Hotwatch},
  Event,
};
use thiserror::Error;

use crate::net::{Client, Request};

#[derive(Debug, Error)]
pub enum WatchError {
  #[error("failed to start watcher: {0}")]
  Hotwatch(#[from] hotwatch::Error),
}

#[derive(Clone, Debug)]
pub struct Config {
  pub sync_path: PathBuf,
  pub device_addrs: Vec<String>,
}

pub fn handle_rename(
  _config: Config,
  _device_clients: HashMap<String, Client>,
  from_path: PathBuf,
  to_path: PathBuf,
) {
  println!("file renamed: from {:?}, to {:?}", from_path, to_path);
}

pub fn handle_create_or_write(
  config: Config,
  mut device_clients: HashMap<String, Client>,
  path: PathBuf,
) {
  println!("file created: {:?}", path);
  let relative_path =
    pathdiff::diff_paths(path.clone(), config.sync_path.clone())
      .expect("failed to diff paths");
  let contents = read_to_string(path).unwrap();
  for client in device_clients.values_mut() {
    let request = Request::put(relative_path.clone(), contents.clone());
    client.request(request).expect("request failed");
  }
}

pub fn handle_remove(
  config: Config,
  mut device_clients: HashMap<String, Client>,
  path: PathBuf,
) {
  println!("file removed: {:?}", path);
  let relative_path = pathdiff::diff_paths(path, config.sync_path.clone())
    .expect("failed to diff paths");
  for client in device_clients.values_mut() {
    let request = Request::del(relative_path.clone());
    client.request(request).expect("request failed");
  }
}

pub fn watch(config: Config) -> Result<(), WatchError> {
  let mut hotwatch = Hotwatch::new()?;
  hotwatch.watch(config.sync_path.clone(), move |event| {
    let mut device_clients = HashMap::new();
    for addr in config.device_addrs.clone() {
      device_clients.insert(addr.clone(), Client::new(addr));
    }
    match event {
      Event::Rename(from_path, to_path) => {
        handle_rename(config.clone(), device_clients, from_path, to_path)
      }
      Event::Create(path) => {
        handle_create_or_write(config.clone(), device_clients, path)
      }
      Event::Write(path) => {
        handle_create_or_write(config.clone(), device_clients, path)
      }
      Event::Remove(path) => {
        handle_remove(config.clone(), device_clients, path)
      }
      _ => {}
    }
    Flow::Continue
  })?;
  hotwatch.run();

  Ok(())
}
