extern crate libc;
extern crate "rust-seccomp" as seccomp;

use std::ptr;
use std::default::Default;

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
  GetRegs = 12,
  Attach = 16,
  Detatch = 17,
  SetOptions = 0x4200
}

#[deriving(Show, FromPrimitive)]
pub enum Event {
  Fork = 1,
  VFork = 2,
  Clone = 3,
  Exec = 4,
  VForkDone = 5,
  Exit = 6,
  Seccomp = 7,
  Stop = 128
}

#[deriving(Default)]
#[deriving(Show)]
pub struct Registers {
  pub r15: i64,
  pub r14: i64,
  pub r13: i64,
  pub r12: i64,
  pub rbp: i64,
  pub rbx: i64,
  pub r11: i64,
  pub r10: i64,
  pub r9: i64,
  pub r8: i64,
  pub rax: i64,
  pub rcx: i64,
  pub rdx: i64,
  pub rsi: i64,
  pub rdi: i64,
  pub orig_rax: i64,
  pub rip: i64,
  pub cs: i64,
  pub eflags: i64,
  pub rsp: i64,
  pub ss: i64,
  pub fs_base: i64,
  pub gs_base: i64,
  pub ds: i64,
  pub es: i64,
  pub fs: i64,
  pub gs: i64
}

bitflags! {
  flags Options: u32 {
    const SysGood = 1,
    const TraceFork = 1 << 1,
    const TraceVFork = 1 << 2,
    const TraceClone = 1 << 3,
    const TraceExec = 1 << 4,
    const TraceVForkDone = 1 << 5,
    const TraceExit = 1 << 6,
    const TraceSeccomp = 1 << 7,
    const ExitKill = 1 << 20
  }
}

pub fn setoptions(pid: libc::pid_t, opts: Options) {
  unsafe {
    raw (Request::SetOptions, pid, ptr::null_mut(), opts.bits as *mut
    libc::c_void);
  }
}

pub fn getregs(pid: libc::pid_t) -> Registers {
  let mut buf: Registers = Default::default();
  let buf_mut: *mut Registers = &mut buf;

  unsafe {
    raw (Request::GetRegs, pid, ptr::null_mut(), buf_mut as *mut libc::c_void);
  }

  return buf;
}

pub fn attach(pid: libc::pid_t) {
  unsafe {
    raw (Request::Attach, pid, ptr::null_mut(), ptr::null_mut());
  }
}

pub fn release(pid: libc::pid_t, signal: int) {
  unsafe {
    raw (Request::Detatch, pid, ptr::null_mut(), signal as *mut libc::c_void);
  }
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

pub fn WIFSTOPPED(s: int) -> bool {
  return (s & 0xff) == 0x7f;
}

pub fn WSTOPSIG(s: int) -> int {
  return (s & 0xff00) >> 8;
}

pub fn WIFCONTINUED(s: int) -> bool {
  return s == 0xffff;
}

pub fn WIFSIGNALED(s: int) -> bool {
  return (((s & 0x7f) + 1) >> 1) > 0;
}

pub fn WIFEXITED(s: int) -> bool {
  return WTERMSIG(s) == 0;
}

pub fn WTERMSIG(s: int) -> int {
  return s & 0x7f;
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

#[deriving(Show)]
pub struct Syscall {
  pub args: [i64, ..6],
  pub call: seccomp::Syscall,
  pub pid: libc::pid_t,
  pub returnVal: i64
}

impl Syscall {
  pub fn from_pid(pid: libc::pid_t) -> Syscall {
    let regs = getregs (pid);
    Syscall {
      pid: pid,
      call: FromPrimitive::from_i64(regs.orig_rax).expect("Unknown syscall"),
      args: [regs.rdi, regs.rsi, regs.rdx, regs.rcx, regs.r8, regs.r9],
      returnVal: 0
    }
  }
}
