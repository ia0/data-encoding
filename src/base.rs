//! Generic base module.
//!
//! This module defines a generic interface, namely
//! [`Base`](trait.Base.html), and an optimized implementation, namely
//! [`Opt`](struct.Opt.html), for positional numerical systems with
//! radix ranging from 2 to 64 (powers of two only). Other base
//! constraints are described in the [`Base`](trait.Base.html)
//! interface.

use std::{error, fmt};
use std::marker::PhantomData;

/// Generic interface.
///
/// A base implementation needs to define at least its padding `pad`
/// and its symbol-to-value function `val`. Its power of two `bit` and
/// its value-to-symbol function `sym` are uniquely defined from the
/// `pad` and `val` functions. However, for performance reasons, all
/// functions may be defined directly.
///
/// # Vocabulary
///
/// We call _ascii_ a 7-bit code represented as a `u8`. In other
/// words, any `u8` value with most significant bit cleared (or
/// equivalently, any `u8` value between 0 and 127 inclusive) is an
/// ascii, and reciprocally.
///
/// We call _value_ a _b_-bit code represented as a `u8`, where _b_ is
/// the power of two of the base, _i.e._ 4 for `base16` and 6 for
/// `base64` for instance. The values for `base64` are thus any `u8`
/// value from 0 to 63, and similarly the values for `base16` are any
/// `u8` value from 0 to 15. More generally, values are any `u8` value
/// between 0 and 2^b - 1 inclusive. Each symbol is uniquely
/// associated to a value.
///
/// We call _symbol_ the symbols represented as ascii (as such, only
/// ascii symbols are allowed). For instance, in `base64`, the symbols
/// are the ascii from `A` to `Z`, the ascii from `a` to `z`, the
/// ascii from `0` to `9`, the ascii `+`, and the ascii `/` in value
/// order. And the `base16` symbols are the ascii from `0` to `9` and
/// the ascii from `A` to `F`.
///
/// We call _padding_ the padding represented as ascii (as such, only
/// ascii padding is allowed). For instance, the ascii `=` is used as
/// the padding for `base64` and `base16`.
///
/// # Constraints
///
/// The base interface comes with invariants which must be satisfied
/// by all implementations. Although it cannot be checked, all
/// implementations must be deterministic, _i.e._ they never return
/// two different outputs for the same input and they never panic
/// (unless specified otherwise). The other constraints are described
/// in the [`ValidError`](enum.ValidError.html) enum and may be
/// checked by the [`valid`](fn.valid.html) function. Implementations
/// should also be pure.
pub trait Base {
    /// Returns the padding.
    fn pad(&self) -> u8;

    /// Returns the value of a symbol.
    ///
    /// This function defines what the symbols are, to which value
    /// they are associated, and the base size:
    ///
    /// - A symbol is an input that does not return `None`.
    /// - The value of a symbol is its associated output.
    /// - The base size is the number of symbols.
    ///
    /// In other words, when `val(s)` returns:
    ///
    /// - `Some(v)`: `s` is a symbol with value `v`.
    /// - `None`: `s` is not a symbol.
    fn val(&self, x: u8) -> Option<u8>;

    /// Returns the power of two of the base.
    fn bit(&self) -> usize {
        let mut n = 0;
        for s in 0..128u8 {
            if self.val(s).is_some() {
                n += 1;
            }
        }
        let mut b = 0;
        while n > 1 {
            n /= 2;
            b += 1;
        }
        return b;
    }

    /// Returns the symbol of a value.
    ///
    /// This function must only be called on values.
    ///
    /// # Panics
    ///
    /// May panic when input is not a value.
    fn sym(&self, x: u8) -> u8 {
        for s in 0..128u8 {
            match self.val(s) {
                Some(v) if v == x => return s,
                _ => (),
            }
        }
        unreachable!();
    }
}

/// Returns the bit-mask of a base.
///
/// The bit-mask of a base is the set of bits used by values. In other
/// words, the bit-mask is `(1 << base.bit()) - 1`.
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn mask<B: Base>(base: &B) -> u8 {
    (1 << base.bit()) - 1
}

/// Returns the period length of a base.
///
/// The period length of a base is the number of significant bits
/// after which the encoding or decoding mechanism loops. In other
/// words, the period length is the least common multiple of 8 and
/// `base.bit()`.
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn len<B: Base>(base: &B) -> usize {
    match base.bit() {
        1 | 2 | 4 => 8,
        3 | 6 => 24,
        5 => 40,
        _ => unreachable!(),
    }
}

/// Returns the encoding length of a base.
///
/// The encoding length of a base is the number of ascii it takes
/// before encoding loops. In other words, the encoding length is
/// `len(base) / 8`.
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn enc<B: Base>(base: &B) -> usize {
    len(base) / 8
}

/// Returns the decoding length of a base.
///
/// The decoding length of a base is the number of symbols it takes
/// before decoding loops. In other words, the decoding length is
/// `len(base) / base.bit()`.
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn dec<B: Base>(base: &B) -> usize {
    len(base) / base.bit()
}

/// Optimized implementation.
///
/// This implementation uses static arrays for constant-time lookup.
/// It also uses a phantom type to enable static dispatch on demand.
pub struct Opt<T> {
    /// Symbol to value association.
    ///
    /// This array must have size 256 and defines `val(s)` as
    /// `Some(val[s])` if `val[s] < 128` and `None` otherwise.
    pub val: &'static [u8],

    /// Value to symbol association.
    ///
    /// This array must have size `1 << b` and defines `sym(v)` as
    /// `sym[v]`.
    pub sym: &'static [u8],

    /// The power of two of the base.
    ///
    /// This value defines `bit()` as `bit`.
    pub bit: u8,

    /// The padding.
    ///
    /// This value defines `pad()` as `pad`.
    pub pad: u8,

    pub _phantom: PhantomData<T>,
}

impl<T> Base for Opt<T> {
    fn bit(&self) -> usize {
        self.bit as usize
    }

    fn pad(&self) -> u8 {
        self.pad
    }

    fn val(&self, x: u8) -> Option<u8> {
        let v = self.val[x as usize];
        if v < 128 { Some(v) } else { None }
    }

    fn sym(&self, x: u8) -> u8 {
        self.sym[x as usize]
    }
}

/// Specification implementation.
///
/// This implementation uses an array of inclusive ranges to easily
/// describe symbol to value assocation. It is not meant for
/// performance but for specification using the
/// [`equal`](fn.equal.html) function.
pub struct Spec {
    /// Symbol to value association.
    ///
    /// This array defines inclusive ranges of symbols in value
    /// order. These ranges must not overlap. For instance, for
    /// `&[(b'0', b'9'), (b'A', b'F')]`, `b'0'` has value 0,
    /// `b'8'` has value 8, and `b'E'` has value 14.
    pub val: &'static [(u8, u8)],

    /// The padding.
    pub pad: u8,
}

impl Base for Spec {
    fn pad(&self) -> u8 {
        self.pad
    }

    fn val(&self, x: u8) -> Option<u8> {
        let mut t = 0;
        for &(l, u) in self.val {
            if l <= x && x <= u {
                return Some(t + x - l);
            }
            t += u - l + 1;
        }
        None
    }
}

/// Validity errors.
///
/// This enum defines the invariants of the [`Base`](trait.Base.html)
/// trait and is returned by the [`valid`](fn.valid.html) function
/// when a constraint check fails.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ValidError {
    /// The base must be a power of two between 2 and 64
    /// inclusive.
    ///
    /// The check `1 <= bit() && bit() <= 6` failed.
    BadBit,

    /// The padding must be an ascii.
    ///
    /// The check `pad() < 128` failed.
    PadNotAscii,

    /// The padding must not be a symbol.
    ///
    /// The check `val(pad()) == None` failed.
    PadSymbol,

    /// Symbols must be ascii.
    ///
    /// The check `val(s) == None || s < 128` failed. In other
    /// words, `s` is a symbol and `s` is not ascii.
    SymNotAscii(u8),

    /// Symbols must be mapped to values.
    ///
    /// The check that if `val(s) == Some(v)` then `v < 1 <<
    /// bit()` failed. In other words, `s` is associated to `v`,
    /// but `v` is not a value.
    NotValue(u8),

    /// Symbols must be uniquely mapped.
    ///
    /// The check that if `val(s) == Some(v)` then `sym(v) == s`
    /// failed. In other words, `s` has value `v` but `v` is not
    /// associated to symbol `s`. The [`val`](trait.Base.html)
    /// function must be injective on symbols. This is checked using
    /// [`sym`](trait.Base.html) because it is the inverse of
    /// [`val`](trait.Base.html) on symbols.
    NotInj(u8),

    /// Symbols must be mapped to all values.
    ///
    /// The check `card(val) == 1 << bit()` failed. The `card`
    /// function returns the number of inputs for which its argument
    /// does not return `None`. In other words, the number of symbols
    /// is not equal to the number of values. The
    /// [`val`](trait.Base.html) function must be surjective on
    /// symbols to values.
    NotSurj,
}

impl fmt::Display for ValidError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ValidError::*;
        match self {
            &BadBit => write!(f, "Size is not 2, 4, 8, 16, 32, or 64."),
            &PadNotAscii => write!(f, "Padding is not ascii."),
            &PadSymbol => write!(f, "Padding is a symbol."),
            &SymNotAscii(s) => write!(f, "Symbol {:?} is not ascii.", s as char),
            &NotValue(s) => write!(f, "Symbol {:?} is not mapped to a value.", s as char),
            &NotInj(s) => write!(f, "Symbol {:?} is not uniquely mapped to its value.", s as char),
            &NotSurj => write!(f, "All values do not have an associated symbol."),
        }
    }
}

impl error::Error for ValidError {
    fn description(&self) -> &str {
        use self::ValidError::*;
        match self {
            &BadBit => "size must be 2, 4, 8, 16, 32, or 64",
            &PadNotAscii => "padding must be ascii",
            &PadSymbol => "padding must not be a symbol",
            &SymNotAscii(_) => "symbols must be ascii",
            &NotValue(_) => "symbols must be mapped to values",
            &NotInj(_) => "symbols must be uniquely mapped",
            &NotSurj => "all values must be mapped",
        }
    }
}

/// Checks whether a base is valid.
///
/// This function checks whether a base satisfies the
/// [`Base`](trait.Base.html) constraints, given that the
/// implementation is deterministic.
pub fn valid<B: Base>(base: &B) -> Result<(), ValidError> {
    use self::ValidError::*;
    check!(BadBit, 1 <= base.bit() && base.bit() <= 6);
    check!(PadNotAscii, base.pad() < 128);
    check!(PadSymbol, base.val(base.pad()) == None);
    let mut card = 0u8;
    for s in 0..128u8 {
        if let Some(v) = base.val(s) {
            check!(NotValue(s), v < 1 << base.bit());
            check!(NotInj(s), base.sym(v) == s);
            card += 1;
        }
        let x = s + 128;
        check!(SymNotAscii(x), base.val(x) == None);
    }
    check!(NotSurj, card == 1 << base.bit());
    Ok(())
}

/// Equality errors.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EqualError {
    /// The two bases differ on a symbol or its associated value.
    Symbol(u8),

    /// The two bases differ on the padding.
    Padding,
}

impl fmt::Display for EqualError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::EqualError::*;
        match self {
            &Symbol(s) => write!(f, "Bases differ on symbol {:?}.", s as char),
            &Padding => write!(f, "Bases differ on padding."),
        }
    }
}

impl error::Error for EqualError {
    fn description(&self) -> &str {
        use self::EqualError::*;
        match self {
            &Symbol(_) => "bases must agree on all symbols",
            &Padding => "bases must agree on padding",
        }
    }
}

/// Checks whether two bases are equal.
///
/// This function checks whether the symbols, their associated value,
/// and the padding of two bases are equal. This is enough if both
/// bases are valid.
pub fn equal<B1: Base, B2: Base>(b1: &B1, b2: &B2) -> Result<(), EqualError> {
    use self::EqualError::*;
    check!(Padding, b1.pad() == b2.pad());
    for s in 0..128u8 {
        check!(Symbol(s), b1.val(s) == b2.val(s));
    }
    Ok(())
}
