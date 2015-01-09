#![feature(unboxed_closures)]
extern crate libc;

pub use sandbox::{Sandbox, Event};
use std::mem;
use std::ptr;
use std::ffi;
use std::str;

mod sandbox;
mod vfs;
mod ptrace;
mod posix;

pub type SandboxPtr = *mut libc::c_void;

#[repr(C)]
#[derive(Copy)]
pub struct sandbox_ops {
    data: *mut libc::c_void,
    event: extern "C" fn(sbox: SandboxPtr, event: Event, data: *mut libc::c_void)
}

#[no_mangle]
pub extern "C" fn sandbox_new (ops: *mut sandbox_ops) -> SandboxPtr {
    let f = unsafe { (*ops).event };
    let data = unsafe { (*ops).data };
    let sbox: Box<Sandbox> = Box::new(Sandbox::new(
        Box::new(move |&:sbox:&Sandbox, e| {
            unsafe {
                f(mem::transmute(sbox), e, data);
            }
        })
    ));
    unsafe {
        mem::transmute (sbox)
    }
}

#[no_mangle]
pub extern "C" fn sandbox_free (sbox: SandboxPtr) {
    let _: Box<Sandbox> = unsafe { mem::transmute (sbox) };
}

#[no_mangle]
pub extern "C" fn sandbox_spawn (sbox: SandboxPtr, argv: *const *const libc::c_char) {
    let mut sbox: Box<Sandbox> = unsafe { mem::transmute (sbox) };
    let mut ptrs: Vec<&str> = Vec::new();
    let mut i = 0;
    loop {
        let s = unsafe { *argv.offset(i) };
        println!("{}: {:?}", i, s);
        if s.is_null() {
            break;
        }
        ptrs.push(unsafe { str::from_c_str(s) });
        i += 1;
    }
    sbox.spawn(ptrs.as_slice());
}

#[no_mangle]
pub extern "C" fn sandbox_tick (sbox: SandboxPtr) {
    let mut sbox: Box<Sandbox> = unsafe { mem::transmute (sbox) };
    sbox.tick();
}
