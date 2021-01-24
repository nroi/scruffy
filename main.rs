extern crate libc;

use libc::c_int;
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn rpmvercmp(a: *const c_char, b: *const c_char) -> i32;
}

fn main() {
    let s1 = CString::new("foo").unwrap();
    let s2 = CString::new("bar").unwrap();
    let d = unsafe { rpmvercmp(s1.as_ptr(), s2.as_ptr()) };
    let e = unsafe { rpmvercmp(s2.as_ptr(), s2.as_ptr()) };
    let f = unsafe { rpmvercmp(s2.as_ptr(), s1.as_ptr()) };

    println!("d = {}", d);
    println!("e = {}", e);
    println!("f = {}", f);

}
