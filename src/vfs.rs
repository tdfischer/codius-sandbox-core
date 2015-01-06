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
  sandbox: &Sandbox
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
    println! ("Tried to open a thing");
  }

  pub fn new(sbox: &mut Sandbox) -> VFS {
    VFS {
      mountpoints: Vec::new(),
      sandbox: sbox
    }
  }
}
