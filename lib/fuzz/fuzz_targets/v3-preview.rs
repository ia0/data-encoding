#![no_main]

use data_encoding::v3_preview::{Bit1, Bit2, Bit3, Bit4, Bit5, Bit6, Encoding, False, True};
use data_encoding_fuzz::{decode_prefix, generate_encoding};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let dyn_base = generate_encoding(&mut data);
    let mut count = 0;
    macro_rules! test {
        ($Bit:ident, $Msb:ident, $Pad:ident, $Wrap:ident, $Ignore:ident) => {
            if let Ok(base) = <&Encoding<$Bit, $Msb, $Pad, $Wrap, $Ignore>>::try_from(&dyn_base) {
                count += 1;
                let encoded = base.encode(data);
                assert_eq!(encoded, dyn_base.encode(data));
                assert_eq!(base.decode(encoded.as_bytes()).unwrap(), data);
                if dyn_base.is_canonical() {
                    let raw = decode_prefix(&dyn_base, &mut data);
                    assert_eq!(base.encode(&raw).as_bytes(), data);
                }
            }
        };
    }
    test!(Bit1, False, False, False, False);
    test!(Bit1, False, False, False, True);
    test!(Bit1, False, False, True, True);
    test!(Bit1, False, True, False, False);
    test!(Bit1, False, True, False, True);
    test!(Bit1, False, True, True, True);
    test!(Bit1, True, False, False, False);
    test!(Bit1, True, False, False, True);
    test!(Bit1, True, False, True, True);
    test!(Bit1, True, True, False, False);
    test!(Bit1, True, True, False, True);
    test!(Bit1, True, True, True, True);
    test!(Bit2, False, False, False, False);
    test!(Bit2, False, False, False, True);
    test!(Bit2, False, False, True, True);
    test!(Bit2, False, True, False, False);
    test!(Bit2, False, True, False, True);
    test!(Bit2, False, True, True, True);
    test!(Bit2, True, False, False, False);
    test!(Bit2, True, False, False, True);
    test!(Bit2, True, False, True, True);
    test!(Bit2, True, True, False, False);
    test!(Bit2, True, True, False, True);
    test!(Bit2, True, True, True, True);
    test!(Bit3, False, False, False, False);
    test!(Bit3, False, False, False, True);
    test!(Bit3, False, False, True, True);
    test!(Bit3, False, True, False, False);
    test!(Bit3, False, True, False, True);
    test!(Bit3, False, True, True, True);
    test!(Bit3, True, False, False, False);
    test!(Bit3, True, False, False, True);
    test!(Bit3, True, False, True, True);
    test!(Bit3, True, True, False, False);
    test!(Bit3, True, True, False, True);
    test!(Bit3, True, True, True, True);
    test!(Bit4, False, False, False, False);
    test!(Bit4, False, False, False, True);
    test!(Bit4, False, False, True, True);
    test!(Bit4, False, True, False, False);
    test!(Bit4, False, True, False, True);
    test!(Bit4, False, True, True, True);
    test!(Bit4, True, False, False, False);
    test!(Bit4, True, False, False, True);
    test!(Bit4, True, False, True, True);
    test!(Bit4, True, True, False, False);
    test!(Bit4, True, True, False, True);
    test!(Bit4, True, True, True, True);
    test!(Bit5, False, False, False, False);
    test!(Bit5, False, False, False, True);
    test!(Bit5, False, False, True, True);
    test!(Bit5, False, True, False, False);
    test!(Bit5, False, True, False, True);
    test!(Bit5, False, True, True, True);
    test!(Bit5, True, False, False, False);
    test!(Bit5, True, False, False, True);
    test!(Bit5, True, False, True, True);
    test!(Bit5, True, True, False, False);
    test!(Bit5, True, True, False, True);
    test!(Bit5, True, True, True, True);
    test!(Bit6, False, False, False, False);
    test!(Bit6, False, False, False, True);
    test!(Bit6, False, False, True, True);
    test!(Bit6, False, True, False, False);
    test!(Bit6, False, True, False, True);
    test!(Bit6, False, True, True, True);
    test!(Bit6, True, False, False, False);
    test!(Bit6, True, False, False, True);
    test!(Bit6, True, False, True, True);
    test!(Bit6, True, True, False, False);
    test!(Bit6, True, True, False, True);
    test!(Bit6, True, True, True, True);
    assert_eq!(count, 1);
});
