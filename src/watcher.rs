use std::{
  collections::HashMap,
  fs::read_to_string,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use hotwatch::{
  blocking::{Flow, Hotwatch},
  Event,
};
use thiserror::Error;

use crate::{
  index::{Index, Operation, OperationKind},
  net::{Client, Request},
};

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
  _index: Arc<Mutex<Index>>,
  _config: Config,
  _device_clients: HashMap<String, Client>,
  from_path: PathBuf,
  to_path: PathBuf,
) {
  println!("file renamed: from {:?}, to {:?}", from_path, to_path);
}

pub fn handle_create_or_write(
  index: Arc<Mutex<Index>>,
  config: Config,
  mut device_clients: HashMap<String, Client>,
  path: PathBuf,
) {
  println!("file created: {:?}", path);
  let relative_path =
    pathdiff::diff_paths(path.clone(), config.sync_path.clone())
      .expect("failed to diff paths");
  let contents = read_to_string(path).unwrap();

  if index
    .lock()
    .unwrap()
    .find(OperationKind::Create {
      path: relative_path.clone(),
      contents: contents.clone(),
    })
    .is_some()
  {
    return;
  }

  for client in device_clients.values_mut() {
    let request = Request::put(relative_path.clone(), contents.clone());
    client.request(request).expect("request failed");
  }

  index.lock().unwrap().push(Operation::create(
    "".to_string(),
    relative_path,
    contents,
  ));
}

pub fn handle_remove(
  index: Arc<Mutex<Index>>,
  config: Config,
  mut device_clients: HashMap<String, Client>,
  path: PathBuf,
) {
  println!("file removed: {:?}", path);
  let relative_path = pathdiff::diff_paths(path, config.sync_path.clone())
    .expect("failed to diff paths");

  if index
    .lock()
    .unwrap()
    .find(OperationKind::Remove {
      path: relative_path.clone(),
    })
    .is_some()
  {
    return;
  }

  for client in device_clients.values_mut() {
    let request = Request::del(relative_path.clone());
    client.request(request).expect("request failed");
  }

  index
    .lock()
    .unwrap()
    .push(Operation::remove("".to_string(), relative_path));
}

pub fn watch(
  index: Arc<Mutex<Index>>,
  config: Config,
) -> Result<(), WatchError> {
  let mut hotwatch = Hotwatch::new()?;
  hotwatch.watch(config.sync_path.clone(), move |event| {
    let mut device_clients = HashMap::new();
    for addr in config.device_addrs.clone() {
      device_clients.insert(addr.clone(), Client::new(addr));
    }
    match event {
      Event::Rename(from_path, to_path) => handle_rename(
        index.clone(),
        config.clone(),
        device_clients,
        from_path,
        to_path,
      ),
      Event::Create(path) => handle_create_or_write(
        index.clone(),
        config.clone(),
        device_clients,
        path,
      ),
      Event::Write(path) => handle_create_or_write(
        index.clone(),
        config.clone(),
        device_clients,
        path,
      ),
      Event::Remove(path) => {
        handle_remove(index.clone(), config.clone(), device_clients, path)
      }
      _ => {}
    }
    Flow::Continue
  })?;

  Ok(hotwatch.run())
}
