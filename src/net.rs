use std::{
  ffi::OsString,
  io::{self, Read, Write},
  net::TcpStream,
  os::unix::ffi::OsStringExt,
  path::PathBuf,
  string::FromUtf8Error,
};

use thiserror::Error;

pub const CODE_PUT: u8 = 1;
pub const CODE_DEL: u8 = 2;

#[derive(Debug, Error)]
pub enum RequestParseError {
  #[error("{0}")]
  Io(#[from] io::Error),
  #[error("{0}")]
  FromUtf8(#[from] FromUtf8Error),
  #[error("code was invalid: {0}")]
  InvalidCode(u8),
}

#[derive(Debug, Error)]
pub enum RequestWriteError {
  #[error("{0}")]
  Io(#[from] io::Error),
  #[error("failed to serialize os string: {0:?}")]
  OsString(OsString),
}

pub enum Request {
  Put { path: PathBuf, contents: String },
  Del { path: PathBuf },
}

impl Request {
  pub fn put(path: PathBuf, contents: String) -> Request {
    Request::Put { path, contents }
  }

  pub fn del(path: PathBuf) -> Request {
    Request::Del { path }
  }

  pub fn parse<R>(reader: &mut R) -> Result<Request, RequestParseError>
  where
    R: Read,
  {
    let mut code_bytes: [u8; 1] = [0];
    reader.read_exact(&mut code_bytes)?;

    match code_bytes {
      [CODE_PUT] => {
        let mut path_len_bytes: [u8; 8] = [0; 8];
        reader.read_exact(&mut path_len_bytes)?;
        let path_len = usize::from_le_bytes(path_len_bytes);

        let mut path_bytes = vec![0; path_len];
        reader.read_exact(path_bytes.as_mut_slice())?;
        let path = PathBuf::from(OsString::from_vec(path_bytes));

        let mut contents_len_bytes: [u8; 8] = [0; 8];
        reader.read_exact(&mut contents_len_bytes)?;
        let contents_len = usize::from_le_bytes(contents_len_bytes);

        let mut contents_bytes = vec![0; contents_len];
        reader.read_exact(contents_bytes.as_mut_slice())?;
        let contents = String::from_utf8(contents_bytes)?;

        Ok(Request::put(path, contents))
      }
      [CODE_DEL] => {
        let mut path_len_bytes: [u8; 8] = [0; 8];
        reader.read_exact(&mut path_len_bytes)?;
        let path_len = usize::from_le_bytes(path_len_bytes);

        let mut path_bytes = vec![0; path_len];
        reader.read_exact(path_bytes.as_mut_slice())?;
        let path = PathBuf::from(OsString::from_vec(path_bytes));

        Ok(Request::del(path))
      }
      [code] => return Err(RequestParseError::InvalidCode(code)),
    }
  }

  pub fn write<W>(self, writer: &mut W) -> Result<(), RequestWriteError>
  where
    W: Write,
  {
    match self {
      Request::Put { path, contents } => {
        let path_string = path
          .as_os_str()
          .to_os_string()
          .into_string()
          .map_err(RequestWriteError::OsString)?;
        writer.write(&[CODE_PUT])?;
        writer.write(&path_string.as_bytes().len().to_le_bytes())?;
        writer.write(path_string.as_bytes())?;
        writer.write(&contents.as_bytes().len().to_le_bytes())?;
        writer.write(&contents.as_bytes())?;
        writer.flush()?;
      }
      Request::Del { path } => {
        let path_string = path
          .as_os_str()
          .to_os_string()
          .into_string()
          .map_err(RequestWriteError::OsString)?;
        writer.write(&[CODE_DEL])?;
        writer.write(&path_string.as_bytes().len().to_le_bytes())?;
        writer.write(path_string.as_bytes())?;
        writer.flush()?;
      }
    }

    Ok(())
  }
}

#[derive(Debug, Error)]
pub enum RequestError {}

pub struct Client {
  addr: String,
  stream: Option<TcpStream>,
}

impl Client {
  pub fn new(addr: String) -> Client {
    Client { addr, stream: None }
  }

  pub fn request(&mut self, request: Request) -> Result<(), RequestWriteError> {
    self.attempt_connect()?;
    let stream = self.stream.as_mut().unwrap();
    request.write(stream)?;
    Ok(())
  }

  fn attempt_connect(&mut self) -> io::Result<()> {
    if self.stream.is_none() {
      self.stream = Some(TcpStream::connect(&self.addr)?);
    }
    Ok(())
  }
}
