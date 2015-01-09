extern crate libc;

use std::os;

pub enum PosixResult<T> {
  Ok(T),
  Error(usize)
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
  pub status: u32
}


impl WaitResult {
    pub fn is_stopped(&self) -> bool {
        (self.status & 0xff) == 0x7f
    }

    pub fn stop_signal(&self) -> u32 {
        (self.status & 0xff00) >> 8
    }

    pub fn is_continued(&self) -> bool {
        self.status == 0xffff
    }

    pub fn is_exited(&self) -> bool {
        self.stop_signal() == 0
    }

    pub fn term_signal(&self) -> u32 {
        self.status & 0x7f
    }

    pub fn is_signaled(&self) -> bool {
        (((self.status & 0x7f) + 1) >> 1) > 0
    }
}
