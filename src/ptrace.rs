extern crate libc;

use std::ptr;

pub enum Action {
  Allow,
  Kill
}

pub enum Request {
  TraceMe = 0,
  PeekText = 1,
  PeekData = 2,
  PeekUser = 3,
  PokeText = 4,
  PokeData = 5,
  PokeUser = 6,
  Continue = 7,
  Kill = 8,
  SingleStep = 9,
  GetRegs = 12
}

#[deriving(Show, FromPrimitive)]
pub enum Event {
  Foo,
  Bar
}

pub fn cont(pid: libc::pid_t, signal: int) {
  unsafe {
    raw (Request::Continue, pid, ptr::null_mut(), signal as *mut libc::c_void);
  }
}

pub fn traceme() {
  unsafe {
    raw (Request::TraceMe, 0, ptr::null_mut(), ptr::null_mut());
  }
}

unsafe fn raw(request: Request,
       pid: libc::pid_t,
       addr: *mut libc::c_void,
       data: *mut libc::c_void) -> libc::c_long {
  ptrace (request as libc::c_int, pid, addr, data)
}

extern {
  fn ptrace(request: libc::c_int,
            pid: libc::pid_t,
            addr: *mut libc::c_void,
            data: *mut libc::c_void) -> libc::c_long;
}
