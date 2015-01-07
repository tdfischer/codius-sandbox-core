extern crate core;
extern crate libc;

pub use sandbox::Sandbox;

mod sandbox;
mod vfs;
mod ptrace;
mod posix;

#[no_mangle]
pub unsafe extern "C" fn this_is_a_rust_function_for_c_api() {
    println! ("Foo!");
}
