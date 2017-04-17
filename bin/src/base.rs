use data_encoding::{BitOrder, Builder, DecodeError};
use error::Error;

pub enum Base {
    NoPad { base: ::data_encoding::NoPad },
    Padded { concat: bool, base: ::data_encoding::Padded },
}

impl Base {
    pub fn encode_len(&self, len: usize) -> usize {
        match self {
            &Base::NoPad { base } => base.encode_len(len),
            &Base::Padded { concat: _, base } => base.encode_len(len),
        }
    }
    pub fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        match self {
            &Base::NoPad { base } => base.encode_mut(input, output),
            &Base::Padded { concat: _, base } => base.encode_mut(input, output),
        }
    }
    pub fn decode_len(&self, len: usize) -> Result<usize, DecodeError> {
        match self {
            &Base::NoPad { base } => base.decode_len(len),
            &Base::Padded { concat: _, base } => base.decode_len(len),
        }
    }
    pub fn decode_mut(&self, input: &[u8], output: &mut [u8])
                  -> Result<usize, DecodeError> {
        match self {
            &Base::NoPad { base } => {
                base.decode_mut(input, output)?;
                base.decode_len(input.len())
            },
            &Base::Padded { concat: false, base } =>
                base.decode_mut(input, output),
            &Base::Padded { concat: true, base } =>
                base.decode_concat_mut(input, output),
        }
    }
    pub fn info(&self) {
        match self {
            &Base::NoPad { base } => {
                println!("symbols: {:?}", base.symbols());
                let (new, old) = base.translate();
                if !new.is_empty() {
                    println!("translate: {{ from: {:?}, to: {:?} }}", new, old);
                }
                println!("bit_order: {:?}", base.bit_order());
                if let Some(ctb) = base.check_trailing_bits() {
                    println!("check_trailing_bits: {}", ctb);
                }
            },
            &Base::Padded { concat: _, base } => {
                Base::NoPad { base: *base.no_pad() }.info();
                println!("padding: {:?}", base.padding() as char);
            },
        }
    }
    pub fn encode_block(&self) -> usize {
        match self {
            &Base::NoPad { base } =>
                match base.bit_width() {
                    1 | 2 | 4 => 1,
                    3 | 6 => 3,
                    5 => 5,
                    _ => unreachable!(),
                },
            &Base::Padded { concat: _, base } =>
                Base::NoPad { base: *base.no_pad() }.encode_block(),
        }
    }
    pub fn decode_block(&self) -> usize {
        match self {
            &Base::NoPad { base } => self.encode_block() * 8 / base.bit_width(),
            &Base::Padded { concat: _, base } =>
                Base::NoPad { base: *base.no_pad() }.decode_block(),
        }
    }
    pub fn concat(&mut self) -> ::Result<()> {
        match self {
            &mut Base::NoPad { base: _ } => return Err(Error::InvalidMode),
            &mut Base::Padded { ref mut concat, base: _ } => *concat = true,
        }
        Ok(())
    }
}

pub fn create(symbols: String, padding: Option<String>,
              translate: Option<String>, ignore_trailing_bits: bool,
              least_significant_bit_first: bool) -> ::Result<Base> {
    let mut builder = Builder::new(symbols.as_bytes());
    match &padding {
        &None => (),
        &Some(ref padding) => {
            check!(Error::ParsePadding, padding.as_bytes().len() == 1);
            builder.padding = Some(padding.as_bytes()[0]);
        },
    }
    builder.check_trailing_bits = !ignore_trailing_bits;
    match translate {
        None => (),
        Some(translate) => {
            let translate = translate.as_bytes();
            check!(Error::InvalidTranslate, translate.len() % 2 == 0);
            let (new, old) = translate.split_at(translate.len() / 2);
            let _ = builder.translate(new, old);
        },
    }
    if least_significant_bit_first {
        builder.bit_order = BitOrder::LeastSignificantFirst;
    }
    if padding.is_none() {
        Ok(Base::NoPad { base: builder.no_pad().map_err(Error::Builder)? })
    } else {
        Ok(Base::Padded { concat: false,
                          base: builder.padded().map_err(Error::Builder)? })
    }
}
