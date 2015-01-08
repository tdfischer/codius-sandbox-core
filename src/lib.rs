extern crate libc;

pub use sandbox::Sandbox;
use std::mem;
use std::ptr;
use std::ffi;
use std::str;

mod sandbox;
mod vfs;
mod ptrace;
mod posix;

pub type sbox_ptr = *const ();

#[no_mangle]
pub unsafe extern "C" fn sandbox_new () -> sbox_ptr {
    let sbox: Box<Sandbox> = box Sandbox::new();
    mem::transmute (sbox)
}

#[no_mangle]
pub unsafe extern "C" fn sandbox_free (sbox: sbox_ptr) {
    let _: Box<Sandbox> = mem::transmute (sbox);
}

#[no_mangle]
pub unsafe extern "C" fn sandbox_spawn (sbox: sbox_ptr, argv: *const *const libc::c_char) {
    let mut sbox: Box<Sandbox> = mem::transmute (sbox);
    let mut ptrs: Vec<&str> = Vec::new();
    let mut i = 0;
    loop {
        let s = *argv.offset(i);
        println!("{}: {}", i, s);
        if (s.is_null()) {
            break;
        }
        ptrs.push(str::from_c_str(s));
        i += 1;
    }
    sbox.spawn(ptrs.as_slice());
}

#[no_mangle]
pub unsafe extern "C" fn sandbox_tick (sbox: sbox_ptr) {
    let mut sbox: Box<Sandbox> = mem::transmute (sbox);
    sbox.tick();
}
