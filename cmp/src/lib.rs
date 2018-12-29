extern crate libc;
use libc::{c_int, c_uchar, size_t};

#[link(name = "ref_gcc", kind = "static")]
extern "C" {
    fn encode_seq_gcc(input: *const c_uchar, len: size_t, output: *mut c_uchar);
    fn encode_par_gcc(input: *const c_uchar, len: size_t, output: *mut c_uchar);
    fn decode_seq_gcc(input: *const c_uchar, len: size_t, output: *mut c_uchar) -> c_int;
    fn decode_par_gcc(input: *const c_uchar, len: size_t, output: *mut c_uchar) -> c_int;
}

#[link(name = "ref_clang", kind = "static")]
extern "C" {
    fn encode_seq_clang(input: *const c_uchar, len: size_t, output: *mut c_uchar);
    fn encode_par_clang(input: *const c_uchar, len: size_t, output: *mut c_uchar);
    fn decode_seq_clang(input: *const c_uchar, len: size_t, output: *mut c_uchar) -> c_int;
    fn decode_par_clang(input: *const c_uchar, len: size_t, output: *mut c_uchar) -> c_int;
}

pub fn base64_encode_seq_gcc(input: &[u8], output: &mut [u8]) {
    unsafe { encode_seq_gcc(input.as_ptr(), input.len(), output.as_mut_ptr()) }
}

pub fn base64_encode_par_gcc(input: &[u8], output: &mut [u8]) {
    unsafe { encode_par_gcc(input.as_ptr(), input.len(), output.as_mut_ptr()) }
}

pub fn base64_decode_seq_gcc(input: &[u8], output: &mut [u8]) -> Result<(), ()> {
    unsafe {
        let code = decode_seq_gcc(input.as_ptr(), input.len(), output.as_mut_ptr());
        if code == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn base64_decode_par_gcc(input: &[u8], output: &mut [u8]) -> Result<(), ()> {
    unsafe {
        let code = decode_par_gcc(input.as_ptr(), input.len(), output.as_mut_ptr());
        if code == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn base64_encode_seq_clang(input: &[u8], output: &mut [u8]) {
    unsafe { encode_seq_clang(input.as_ptr(), input.len(), output.as_mut_ptr()) }
}

pub fn base64_encode_par_clang(input: &[u8], output: &mut [u8]) {
    unsafe { encode_par_clang(input.as_ptr(), input.len(), output.as_mut_ptr()) }
}

pub fn base64_decode_seq_clang(input: &[u8], output: &mut [u8]) -> Result<(), ()> {
    unsafe {
        let code = decode_seq_clang(input.as_ptr(), input.len(), output.as_mut_ptr());
        if code == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn base64_decode_par_clang(input: &[u8], output: &mut [u8]) -> Result<(), ()> {
    unsafe {
        let code = decode_par_clang(input.as_ptr(), input.len(), output.as_mut_ptr());
        if code == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
