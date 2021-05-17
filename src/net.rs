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

pub enum Request {
  Put { path: PathBuf, contents: String },
  Del { path: PathBuf },
}

impl Request {
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

  pub fn put(path: PathBuf, contents: String) -> Request {
    Request::Put { path, contents }
  }

  pub fn del(path: PathBuf) -> Request {
    Request::Del { path }
  }
}

pub struct Client {
  addr: String,
  stream: Option<TcpStream>,
}

impl Client {
  pub fn new(addr: String) -> Client {
    Client { addr, stream: None }
  }

  pub fn make_request(&mut self, request: Request) -> io::Result<()> {
    self.attempt_connect()?;
    let stream = self.stream.as_mut().unwrap();

    match request {
      Request::Put { path, contents } => {
        let path_string =
          path.as_os_str().to_os_string().into_string().unwrap();
        stream.write(&[CODE_PUT])?;
        stream.write(&path_string.as_bytes().len().to_le_bytes())?;
        stream.write(path_string.as_bytes())?;
        stream.write(&contents.as_bytes().len().to_le_bytes())?;
        stream.write(&contents.as_bytes())?;
        stream.flush()?;
      }
      Request::Del { path } => {
        let path_string =
          path.as_os_str().to_os_string().into_string().unwrap();
        stream.write(&[CODE_DEL])?;
        stream.write(&path_string.as_bytes().len().to_le_bytes())?;
        stream.write(path_string.as_bytes())?;
        stream.flush()?;
      }
    }

    Ok(())
  }

  pub fn addr(&self) -> &str {
    self.addr.as_str()
  }

  fn attempt_connect(&mut self) -> io::Result<()> {
    self.stream = Some(TcpStream::connect(&self.addr)?);
    Ok(())
  }
}
