#![feature(lang_items, alloc_error_handler)]
#![no_std]
#![no_main]

extern crate data_encoding;
extern crate libc;

use core::fmt::Write;
use data_encoding::{Encoding, BASE32, BASE64, BASE64_NOPAD, HEXLOWER_PERMISSIVE};

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

struct Fd(libc::c_int);

impl core::fmt::Write for Fd {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            libc::write(self.0, s.as_ptr() as *const libc::c_void, s.len());
        }
        Ok(())
    }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let _ = writeln!(Fd(2), "{}", info);
        libc::exit(1);
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use core::alloc::{GlobalAlloc, Layout};

    struct Malloc;

    unsafe impl GlobalAlloc for Malloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            libc::malloc(layout.size()) as *mut u8
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
            libc::free(ptr as *mut libc::c_void)
        }
    }

    #[alloc_error_handler]
    fn foo(_: Layout) -> ! {
        loop {}
    }

    #[global_allocator]
    static GLOBAL: Malloc = Malloc;
}

fn test_encode(encoding: &Encoding, input: &[u8], output: &mut [u8], result: &str) {
    let olen = encoding.encode_len(input.len());
    encoding.encode_mut(input, &mut output[.. olen]);
    assert_eq!(core::str::from_utf8(&output[.. olen]).unwrap(), result);
    #[cfg(feature = "alloc")]
    assert_eq!(encoding.encode(input), result);
}

fn test_decode(encoding: &Encoding, input: &str, output: &mut [u8], result: &[u8]) {
    let ilen = encoding.decode_len(input.len()).unwrap();
    let olen = encoding.decode_mut(input.as_bytes(), &mut output[.. ilen]).unwrap();
    assert_eq!(&output[.. olen], result);
    #[cfg(feature = "alloc")]
    assert_eq!(encoding.decode(input.as_bytes()).unwrap(), result);
}

fn test(encoding: &Encoding, input: &[u8], output: &str, buffer: &mut [u8]) {
    test_encode(encoding, input, buffer, output);
    test_decode(encoding, output, buffer, input);
}

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    test(&BASE32, b"hello", "NBSWY3DP", &mut [0; 8]);
    test(&BASE64, b"hello", "aGVsbG8=", &mut [0; 8]);
    test(&BASE64_NOPAD, b"hello", "aGVsbG8", &mut [0; 8]);
    test(&HEXLOWER_PERMISSIVE, b"hello", "68656c6c6f", &mut [0; 10]);
    test_decode(&HEXLOWER_PERMISSIVE, "68656C6C6F", &mut [0; 5], b"hello");
    let _ = writeln!(Fd(1), "All tests passed.");
    0
}
