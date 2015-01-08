extern crate libc;

pub use sandbox::Sandbox;
use std::mem;

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
pub unsafe extern "C" fn sandbox_spawn (sbox: sbox_ptr, argv: &[str]) {
    println! ("Spawning!");
    let mut sbox: Box<Sandbox> = mem::transmute (sbox);
    sbox.spawn(&["/bin/true"]);
}

#[no_mangle]
pub unsafe extern "C" fn sandbox_tick (sbox: sbox_ptr) {
    let mut sbox: Box<Sandbox> = mem::transmute (sbox);
    sbox.tick();
}
