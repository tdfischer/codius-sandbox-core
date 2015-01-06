extern crate libc;

use std::os;

pub enum PosixResult<T> {
  Ok(T),
  Error(uint)
}

impl <T> PosixResult<T> {
  pub fn expect(self, msg: &str) -> T {
    match self {
      PosixResult::Ok(value) => value,
      PosixResult::Error(errno) => panic!("Error: {}", os::error_string (errno))
    }
  }
}

pub struct WaitResult {
  pub pid: libc::pid_t,
  pub status: int
}

