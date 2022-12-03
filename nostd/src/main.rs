#![feature(lang_items, default_alloc_error_handler)]
#![no_std]
#![no_main]

use core::fmt::Write;

use data_encoding::Encoding;

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
        let _ = writeln!(Fd(2), "{info}");
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

fn test_macro() {
    const FOOBAR: &[u8] = &data_encoding_macro::base64!("Zm9vYmFy");
    const LETTER8: Encoding = data_encoding_macro::new_encoding! {
        symbols: "ABCDEFGH",
    };

    assert_eq!(FOOBAR, b"foobar");
    test(&LETTER8, &[0], "AAA", &mut [0; 3]);
    test(&LETTER8, b"foobar", "DBEGHFFHDAEGAFGC", &mut [0; 16]);
}

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    test(&data_encoding::BASE32, b"hello", "NBSWY3DP", &mut [0; 8]);
    test(&data_encoding::BASE64, b"hello", "aGVsbG8=", &mut [0; 8]);
    test(&data_encoding::BASE64_NOPAD, b"hello", "aGVsbG8", &mut [0; 8]);
    test(&data_encoding::HEXLOWER_PERMISSIVE, b"hello", "68656c6c6f", &mut [0; 10]);
    test_decode(&data_encoding::HEXLOWER_PERMISSIVE, "68656C6C6F", &mut [0; 5], b"hello");
    test_macro();
    let _ = writeln!(Fd(1), "All tests passed.");
    0
}
