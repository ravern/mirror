use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum OperationKind {
  Create {
    path: PathBuf,
    contents: String, // TODO: Use a hash of contents instead.
  },
  Remove {
    path: PathBuf,
  },
}

pub struct Index {
  ops: Vec<Operation>,
}

impl Index {
  pub fn new() -> Index {
    Index { ops: vec![] }
  }

  pub fn push(&mut self, op: Operation) {
    self.ops.push(dbg!(op));
  }

  pub fn find(&self, op_kind: OperationKind) -> Option<&Operation> {
    self.ops.iter().find(|op| op.kind == op_kind)
  }
}

#[derive(Debug)]
pub struct Operation {
  device_addr: String,
  kind: OperationKind,
}

impl Operation {
  pub fn create(
    device_addr: String,
    path: PathBuf,
    contents: String,
  ) -> Operation {
    Operation {
      device_addr,
      kind: OperationKind::Create { path, contents },
    }
  }

  pub fn remove(device_addr: String, path: PathBuf) -> Operation {
    Operation {
      device_addr,
      kind: OperationKind::Remove { path },
    }
  }
}
