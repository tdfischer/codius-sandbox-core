extern crate libc;
extern crate seccomp;

use std::os;
use std::ptr;
use std::any;

mod ptrace;

pub enum PosixResult<T> {
  Ok(T),
  Error(uint)
}

impl <T> PosixResult<T> {
  fn expect(self, msg: &str) -> T {
    match self {
      PosixResult::Ok(value) => value,
      PosixResult::Error(errno) => panic!("Error: {}", os::error_string (errno))
    }
  }
}

pub struct WaitResult {
  pid: libc::pid_t,
  status: int
}

pub struct Sandbox {
  pub pid: libc::pid_t
}

impl Sandbox {
  pub fn spawn(&mut self, argv: &[&str]) {
    self.pid = unsafe { fork() };
    match self.pid {
      0 => self.exec_child(argv),
      _ => self.trace_child()
    }
  }

  fn trace_child(&self) {
    println! ("Tracing {}", self.pid);
    loop {
      let res = self.wait_on_child().expect ("Could not call waitpid");

      let st = ((res.status >> 8) & !5) >> 8;

      let status: ptrace::Event =
          FromPrimitive::from_i64(st as i64).expect("Unknown status");

      println! ("Got activity from child {}: {} ({})", res.pid, res.status, st);
      ptrace::cont (self.pid, 0);
    }
  }

  fn exec_child(&self, argv: &[&str]) -> ! {
    ptrace::traceme();
    unsafe {
      raise (19);
    }
    let command = argv[0].to_c_str();
    let mut ptrs : Vec<*const libc::c_char> = Vec::with_capacity(argv.len());
    for arg in argv.iter() {
      ptrs.push (arg.to_c_str().as_ptr());
    }
    ptrs.push (ptr::null());
    println! ("Calling execvp...");

    let filter = seccomp::SeccompFilter::new(seccomp::Action::Trace(0));
    filter.load();

    unsafe {
      libc::execvp (command.as_ptr(), ptrs.as_mut_ptr());
    }
    panic!("Could not fork, got: {} - {}", os::errno(), os::last_os_error());
  }

  pub fn new() -> Sandbox {
    Sandbox { pid: -1 }
  }

  pub fn wait_on_child(&self) -> PosixResult<WaitResult> {
    let pid;
    let mut st: libc::c_int = 0;
    unsafe {
      pid = waitpid (self.pid, &mut st, 0);
    }

    if pid > 0 {
      PosixResult::Ok (WaitResult {
        pid: pid,
        status: st as int
      })
    } else {
      PosixResult::Error (os::errno())
    }
  }
}

extern {
  fn raise(signum: libc::c_int) -> libc::c_int;
  fn fork() -> libc::pid_t;
  fn waitpid(pid: libc::pid_t, status: *mut libc::c_int, options: libc::c_int) -> libc::pid_t;
}
