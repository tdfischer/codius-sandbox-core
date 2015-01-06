#![feature(macro_rules)]
extern crate libc;
extern crate "rust-seccomp" as seccomp;

use std::vec::Vec;
use ptrace;

pub struct Filesystem {
  foo: int
}

pub struct VFS {
  mountpoints: Vec<Filesystem>,
}

impl VFS {
  pub fn handle_syscall(&self, call: ptrace::Syscall) -> ptrace::Syscall{
    match (call.call) {
      seccomp::Syscall::OPEN => self.do_open(&call),
      _ => ()
    }
    return call;
  }

  fn do_open(&self, call: &ptrace::Syscall) {
    let reader = ptrace::Reader::new(call.pid);
    let fname = reader.read_string(call.args[0]);
    println! ("Tried to open a thing: {}", String::from_utf8(fname).unwrap());
  }

  pub fn new() -> VFS {
    VFS {
      mountpoints: Vec::new(),
    }
  }
}
