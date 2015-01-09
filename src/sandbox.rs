extern crate libc;
extern crate "rust-seccomp" as seccomp;

use ptrace;
use vfs;
use posix::{PosixResult, WaitResult};
use std::os;
use std::ptr;
use std::num::FromPrimitive;
use std::ffi::CString;

#[derive(Show)]
pub enum Event {
    Released,
    Exited
}

pub struct Sandbox {
  pub pid: libc::pid_t,
  pub running: bool,
  pub vfs: vfs::VFS,
  cb_event: Box<Fn(&Sandbox, Event) + 'static>
}

impl Sandbox {
    pub fn event(&self, event: Event) {
        let h = self.cb_event;
        h(self, event);
    }

  pub fn spawn(&mut self, argv: &[&str]) {
    self.running = true;
    self.pid = unsafe { fork() };
    match self.pid {
      0 => self.exec_child(argv),
      _ => self.trace_child()
    }
  }

  fn trace_child(&self) {
    println! ("Tracing {}", self.pid);

    ptrace::attach (self.pid);
    self.wait_on_child().expect ("Could not call waitpid");
    ptrace::setoptions (self.pid, ptrace::TraceExit | ptrace::ExitKill |
    ptrace::TraceSeccomp | ptrace::TraceExec | ptrace::TraceClone);
    ptrace::cont (self.pid, 0);
  }

  pub fn tick(&mut self) {
    let res = self.wait_on_child().expect ("Could not call waitpid");

    if res.is_stopped() {
      if res.stop_signal() == 5 {
        let st = ((res.status >> 8) & !5) >> 8;

        let event: ptrace::Event =
            FromPrimitive::from_i64(st as i64).expect("Unknown status");

        let regs = ptrace::getregs (res.pid);

        match event {
          ptrace::Event::Seccomp => self.handle_seccomp (res.pid),
          ptrace::Event::Exit => self.handle_exit (res.pid),
          _ => ptrace::cont (res.pid, 0)
        }
      } else {
        println! ("Got stop signal {}", res.stop_signal());
        ptrace::cont (self.pid, res.stop_signal());
      }
    } else if res.is_signaled() {
      println! ("Killed by {}", res.term_signal());
      panic! ("Child died by a signal");
    } else if res.is_exited() {
      panic! ("Child exited.");
    }
  }

  fn handle_exit(&mut self, pid: libc::pid_t) {
    println! ("Child exited cleanly. Maybe.");
    self.event(Event::Exited);
    self.release_child(0);
  }

  fn release_child(&mut self, signal: i32) {
    ptrace::release (self.pid, signal);
    self.event(Event::Released);
    self.running = false;
  }

  fn handle_seccomp(&self, pid: libc::pid_t) {
    let regs = ptrace::getregs (pid);
    let mut call = ptrace::Syscall::from_pid (pid);
    //println! ("Attempted syscall {:?}", call);
    call = self.vfs.handle_syscall (call);
    ptrace::cont (pid, 0);
  }

  fn exec_child(&self, argv: &[&str]) -> ! {
    ptrace::traceme();
    unsafe {
      raise (19);
    }
    let command = CString::from_slice(argv[0].as_bytes());
    let mut ptrs : Vec<*const libc::c_char> = Vec::with_capacity(argv.len());
    for arg in argv.iter() {
      ptrs.push (CString::from_slice(arg.as_bytes()).as_ptr());
    }
    ptrs.push (ptr::null());

    let filter = seccomp::Filter::new(&seccomp::ACT_KILL);
    let trace = seccomp::act_trace(0);

    // A duplicate in case the seccomp_init() call is accidentally modified
    filter.rule_add(&seccomp::ACT_KILL, seccomp::Syscall::PTRACE, &[]);

    // This is actually caught via PTRACE_EVENT_EXEC
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EXECVE, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::CLONE, &[]);

    // Use to track chdir calls
    filter.rule_add(&trace, seccomp::Syscall::CHDIR, &[]);
    filter.rule_add(&trace, seccomp::Syscall::FCHDIR, &[]);

    // These interact with the VFS layer
    filter.rule_add(&trace, seccomp::Syscall::OPEN, &[]);
    filter.rule_add(&trace, seccomp::Syscall::ACCESS, &[]);
    filter.rule_add(&trace, seccomp::Syscall::OPENAT, &[]);
    filter.rule_add(&trace, seccomp::Syscall::STAT, &[]);
    filter.rule_add(&trace, seccomp::Syscall::LSTAT, &[]);
    filter.rule_add(&trace, seccomp::Syscall::GETCWD, &[]);
    filter.rule_add(&trace, seccomp::Syscall::READLINK, &[]);

    macro_rules! vfs_filter(
      ($call:ident) => ({
        filter.rule_add(&trace, seccomp::Syscall::$call, &[
          seccomp::Compare::new(0, seccomp::Op::OpGe, 4098)
        ]);
        filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::$call, &[
          seccomp::Compare::new(0, seccomp::Op::OpLt, 4098)
        ]);
      });
    );


    vfs_filter!(READ);
    vfs_filter!(CLOSE);
    vfs_filter!(IOCTL);
    vfs_filter!(FSTAT);
    vfs_filter!(LSEEK);
    vfs_filter!(WRITE);
    vfs_filter!(GETDENTS);
    //vfs_filter!(READDIR);
    vfs_filter!(GETDENTS64);
    vfs_filter!(READV);
    vfs_filter!(WRITEV);

    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::FSYNC, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::FDATASYNC, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::SYNC, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::POLL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::MMAP, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::MPROTECT, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::MUNMAP, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::MADVISE, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::BRK, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::RT_SIGACTION, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::RT_SIGPROCMASK, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::SELECT, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::SCHED_YIELD, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::GETPID, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::ACCEPT, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::LISTEN, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EXIT, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::GETTIMEOFDAY, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::TKILL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EPOLL_CREATE, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::RESTART_SYSCALL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::CLOCK_GETTIME, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::CLOCK_GETRES, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::CLOCK_NANOSLEEP, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::GETTID, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::IOCTL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::NANOSLEEP, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EXIT_GROUP, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EPOLL_WAIT, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EPOLL_CTL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::TGKILL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::PSELECT6, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::PPOLL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::ARCH_PRCTL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::PRCTL, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::SET_ROBUST_LIST, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::GET_ROBUST_LIST, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EPOLL_PWAIT, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::ACCEPT4, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EVENTFD2, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::EPOLL_CREATE1, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::PIPE2, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::FUTEX, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::SET_TID_ADDRESS, &[]);
    filter.rule_add(&seccomp::ACT_ALLOW, seccomp::Syscall::SET_THREAD_AREA, &[]);

    filter.load();

    println! ("Exec {:?}", command);

    unsafe {
      libc::execvp (command.as_ptr(), ptrs.as_mut_ptr());
    }

    panic!("Could not fork, got: {} - {}", os::errno(), os::last_os_error());
  }

  pub fn new<H>(handler: H) -> Self where H: Fn(&Sandbox, Event) + 'static {
    Sandbox {
      pid: -1,
      running: false,
      vfs: vfs::VFS::new(),
      cb_event: Box::new(handler)
    }
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
        status: st as u32
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
