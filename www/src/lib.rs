#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate data_encoding;
#[macro_use]
extern crate lazy_static;
extern crate wasm_bindgen;

mod range;
mod state;
mod utf8;

use data_encoding::{BASE64URL_NOPAD, Encoding, Specification};
use std::collections::HashMap;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    fn body() -> JsValue;
    fn createElement(name: &str) -> JsValue;
    fn createTextNode(text: &str) -> JsValue;
    fn appendChild(parent: &JsValue, child: &JsValue);
    fn insertBefore(parent: &JsValue, child: &JsValue, node: &JsValue);
    fn removeChild(parent: &JsValue, child: &JsValue);
    fn setAttribute(node: &JsValue, name: &str, value: &str);
    fn getElementById(id: &str) -> JsValue;
    fn value(node: &JsValue) -> String;
    fn set_value(node: &JsValue, value: &str);
    fn addClass(node: &JsValue, name: &str);
    fn removeClass(node: &JsValue, name: &str);
    fn is_checked(node: &JsValue) -> bool;
    fn set_checked(node: &JsValue);
    fn setStorage(name: &str, value: &str);
    fn getStorage(name: &str) -> JsValue;
    fn deleteStorage(name: &str);
    fn clearStorage();
    fn setHistory(name: &str, value: &str);
    fn getHistory(name: &str) -> JsValue;
    fn deleteHistory(name: &str);
}

lazy_static! {
    static ref PRESETS: HashMap<String, Option<Encoding>> = {
        let mut map = HashMap::new();
        map.insert("no encoding".to_string(), None);
        macro_rules! add {
            ($b:ident) => {
                map.insert(stringify!($b).to_string(), Some(data_encoding::$b))
            };
        }
        add!(BASE32);
        add!(BASE64);
        add!(BASE32HEX);
        add!(BASE32HEX_NOPAD);
        add!(BASE32_DNSCURVE);
        add!(BASE32_DNSSEC);
        add!(BASE32_NOPAD);
        add!(BASE64URL);
        add!(BASE64URL_NOPAD);
        add!(BASE64_MIME);
        add!(BASE64_NOPAD);
        add!(HEXLOWER);
        add!(HEXLOWER_PERMISSIVE);
        add!(HEXUPPER);
        add!(HEXUPPER_PERMISSIVE);
        map
    };
}

fn create_tooltip(text: &str, tooltip: &str) -> JsValue {
    let node = createElement("div");
    setAttribute(&node, "class", "tooltip");
    let tooltip_node = createElement("span");
    setAttribute(&tooltip_node, "class", "tooltiptext");
    appendChild(&tooltip_node, &createTextNode(tooltip));
    appendChild(&node, &tooltip_node);
    appendChild(&node, &createTextNode(text));
    node
}

fn create_option(value: &str) -> JsValue {
    let node = createElement("option");
    setAttribute(&node, "value", value);
    appendChild(&node, &createTextNode(value));
    node
}

fn create_switch(name: &str, id: i32) -> JsValue {
    let switch = createElement("input");
    setAttribute(&switch, "type", "radio");
    setAttribute(&switch, "name", &format!("{}_{}", name, id));
    let spec_update = format!("wasm_bindgen.spec_update({})", id);
    setAttribute(&switch, "oninput", &spec_update);
    switch
}

fn create_menu(id: i32) -> JsValue {
    let menu = createElement("div");
    setAttribute(&menu, "class", "menu");

    let swap_left = createElement("button");
    setAttribute(&swap_left, "type", "button");
    let swap_left_onclick = format!("wasm_bindgen.swap_left({})", id);
    setAttribute(&swap_left, "onclick", &swap_left_onclick);
    appendChild(&swap_left, &createTextNode("<"));
    appendChild(&menu, &swap_left);

    let delete = createElement("button");
    setAttribute(&delete, "type", "button");
    let delete_onclick = format!("wasm_bindgen.delete_encoding({})", id);
    setAttribute(&delete, "onclick", &delete_onclick);
    appendChild(&delete, &createTextNode("\u{d7}"));
    appendChild(&menu, &delete);

    let swap_right = createElement("button");
    setAttribute(&swap_right, "type", "button");
    let swap_right_onclick = format!("wasm_bindgen.swap_right({})", id);
    setAttribute(&swap_right, "onclick", &swap_right_onclick);
    appendChild(&swap_right, &createTextNode(">"));
    appendChild(&menu, &swap_right);

    menu
}

fn create_specification(id: i32) -> JsValue {
    let specification = createElement("div");
    setAttribute(&specification, "id", &format!("spec_{}", id));
    setAttribute(&specification, "class", "specification");
    let spec_update = format!("wasm_bindgen.spec_update({})", id);

    let preset = createElement("select");
    let load_preset = format!("wasm_bindgen.load_preset({})", id);
    setAttribute(&preset, "id", &format!("preset_{}", id));
    setAttribute(&preset, "onchange", &load_preset);
    let mut presets: Vec<_> = PRESETS.keys().collect();
    presets.sort();
    for x in presets {
        appendChild(&preset, &create_option(x));
    }
    appendChild(&specification, &createTextNode("Preset"));
    appendChild(&specification, &preset);
    set_value(&preset, "");

    // symbols
    let symbols_tooltip = "The number of symbols must be 2, 4, 8, 16, 32, or 64. Symbols must be \
                           ASCII characters (smaller than 128) and they must be unique.";
    appendChild(&specification, &create_tooltip("Symbols", symbols_tooltip));
    let symbols = createElement("input");
    setAttribute(&symbols, "type", "text");
    setAttribute(&symbols, "placeholder", "no encoding");
    setAttribute(&symbols, "id", &format!("symbols_{}", id));
    setAttribute(&symbols, "oninput", &spec_update);
    appendChild(&specification, &symbols);

    // bit order: LSB | MSB
    let bit_order_tooltip =
        "The default is to use most significant bit first since it is the most common.";
    appendChild(
        &specification,
        &create_tooltip("Bit order", bit_order_tooltip),
    );
    let bit_order = createElement("div");
    setAttribute(&bit_order, "class", "switch");
    let lsb = create_switch("bit_order", id);
    setAttribute(&lsb, "id", &format!("bit_order_lsb_{}", id));
    appendChild(&bit_order, &createTextNode("LSB first"));
    appendChild(&bit_order, &lsb);
    let msb = create_switch("bit_order", id);
    setAttribute(&msb, "id", &format!("bit_order_msb_{}", id));
    setAttribute(&msb, "checked", "");
    appendChild(&bit_order, &msb);
    appendChild(&bit_order, &createTextNode("MSB first"));
    appendChild(&specification, &bit_order);

    // trailing bits: check | ignore
    let trailing_bits_tooltip = "The default is to check trailing bits. This is ignored when \
                                 unnecessary (i.e. for base2, base4, and base16).";
    appendChild(
        &specification,
        &create_tooltip("Trailing bits", trailing_bits_tooltip),
    );
    let trailing_bits = createElement("div");
    setAttribute(&trailing_bits, "class", "switch");
    let check = create_switch("trailing_bits", id);
    setAttribute(&check, "id", &format!("trailing_bits_check_{}", id));
    setAttribute(&check, "checked", "");
    appendChild(&trailing_bits, &createTextNode("check"));
    appendChild(&trailing_bits, &check);
    let ignore = create_switch("trailing_bits", id);
    setAttribute(&ignore, "id", &format!("trailing_bits_ignore_{}", id));
    appendChild(&trailing_bits, &ignore);
    appendChild(&trailing_bits, &createTextNode("ignore"));
    appendChild(&specification, &trailing_bits);

    // padding
    let padding_tooltip = "The padding character must be ASCII and must not be a symbol.";
    appendChild(&specification, &create_tooltip("Padding", padding_tooltip));
    let padding = createElement("input");
    setAttribute(&padding, "type", "text");
    setAttribute(&padding, "placeholder", "no padding");
    setAttribute(&padding, "id", &format!("padding_{}", id));
    setAttribute(&padding, "oninput", &spec_update);
    appendChild(&specification, &padding);

    // ignore
    let ignore_tooltip =
        "The characters to ignore must be ASCII and must not be symbols or the padding character.";
    appendChild(&specification, &create_tooltip("Ignore", ignore_tooltip));
    let ignore = createElement("input");
    setAttribute(&ignore, "type", "text");
    setAttribute(&ignore, "placeholder", "no characters ignored");
    setAttribute(&ignore, "id", &format!("ignore_{}", id));
    setAttribute(&ignore, "oninput", &spec_update);
    appendChild(&specification, &ignore);

    // wrap width
    let wrap_width_tooltip = "Must be a multiple of: 8 for base2, base8, and base32; 4 for base4 \
                              and base64; 2 for base16.";
    appendChild(
        &specification,
        &create_tooltip("Wrap width", wrap_width_tooltip),
    );
    let wrap_width = createElement("input");
    setAttribute(&wrap_width, "type", "text");
    setAttribute(&wrap_width, "placeholder", "no wrapping");
    setAttribute(&wrap_width, "id", &format!("wrap_width_{}", id));
    setAttribute(&wrap_width, "oninput", &spec_update);
    appendChild(&specification, &wrap_width);

    // wrap separator
    let wrap_separator_tooltip =
        "The wrapping characters must be ASCII and must not be symbols or the padding character.";
    appendChild(
        &specification,
        &create_tooltip("Wrap separator", wrap_separator_tooltip),
    );
    let wrap_separator = createElement("input");
    setAttribute(&wrap_separator, "type", "text");
    setAttribute(&wrap_separator, "placeholder", "no wrapping");
    setAttribute(&wrap_separator, "id", &format!("wrap_separator_{}", id));
    setAttribute(&wrap_separator, "oninput", &spec_update);
    appendChild(&specification, &wrap_separator);

    // translate from
    let translate_from_tooltip = "The characters to translate from must be ASCII and must not \
                                  have already been assigned a semantics.";
    appendChild(
        &specification,
        &create_tooltip("Translate from", translate_from_tooltip),
    );
    let translate_from = createElement("input");
    setAttribute(&translate_from, "type", "text");
    setAttribute(&translate_from, "placeholder", "no translation");
    setAttribute(&translate_from, "id", &format!("translate_from_{}", id));
    setAttribute(&translate_from, "oninput", &spec_update);
    appendChild(&specification, &translate_from);

    // translate to
    let translate_to_tooltip = "The characters to translate to must be ASCII and must have been \
                                assigned a semantics (symbol, padding character, or ignored \
                                character).";
    appendChild(
        &specification,
        &create_tooltip("Translate to", translate_to_tooltip),
    );
    let translate_to = createElement("input");
    setAttribute(&translate_to, "type", "text");
    setAttribute(&translate_to, "placeholder", "no translation");
    setAttribute(&translate_to, "id", &format!("translate_to_{}", id));
    setAttribute(&translate_to, "oninput", &spec_update);
    appendChild(&specification, &translate_to);

    // canonical
    let canonical_tooltip = "The encoding is not canonical if trailing bits are not checked, \
                             padding is used, characters are ignored, or characters are \
                             translated.";
    appendChild(
        &specification,
        &create_tooltip("Canonical", canonical_tooltip),
    );
    let canonical = createElement("output");
    setAttribute(&canonical, "id", &format!("canonical_{}", id));
    appendChild(&specification, &canonical);

    specification
}

fn create_encoding(id: i32) -> JsValue {
    let encoding = createElement("div");
    setAttribute(&encoding, "id", &format!("encoding_{}", id));
    setAttribute(&encoding, "class", "encoding");

    let menu = create_menu(id);
    appendChild(&encoding, &menu);

    let text = createElement("textarea");
    setAttribute(&text, "id", &format!("text_{}", id));
    let text_update = format!("wasm_bindgen.text_update({})", id);
    setAttribute(&text, "rows", "5");
    setAttribute(&text, "cols", "50");
    setAttribute(&text, "placeholder", "enter your text here");
    setAttribute(&text, "oninput", &text_update);
    setAttribute(&text, "onfocus", &text_update);
    appendChild(&encoding, &text);

    let specification = create_specification(id);
    appendChild(&encoding, &specification);

    let error = createElement("output");
    setAttribute(&error, "id", &format!("error_{}", id));
    setAttribute(&error, "class", "error");
    appendChild(&encoding, &error);

    encoding
}

fn get_encoding(id: i32) -> Result<Option<Encoding>, String> {
    let utf8_decode = |name| -> Result<_, String> {
        let value = value(&getElementById(&format!("{}_{}", name, id)));
        Ok(String::from_utf8_lossy(&utf8::decode(&value)?).into_owned())
    };
    let symbols = utf8_decode("symbols")?;
    if symbols.is_empty() {
        return Ok(None);
    }
    let mut spec = Specification::new();
    spec.symbols = range::decode(&symbols)?;
    if is_checked(&getElementById(&format!("bit_order_lsb_{}", id))) {
        spec.bit_order = data_encoding::BitOrder::LeastSignificantFirst;
    }
    if is_checked(&getElementById(&format!("trailing_bits_ignore_{}", id))) {
        spec.check_trailing_bits = false;
    }
    let padding = utf8_decode("padding")?;
    let mut padding_iter = padding.chars().fuse();
    spec.padding = padding_iter.next();
    if padding_iter.next().is_some() {
        return Err("padding has more than one character".to_string());
    }
    match spec.symbols.len() {
        2 | 4 | 16 => spec.padding = None,
        _ => (),
    }
    spec.ignore = range::decode(&utf8_decode("ignore")?)?;
    let wrap_width = value(&getElementById(&format!("wrap_width_{}", id)));
    if !wrap_width.is_empty() {
        match wrap_width.parse() {
            Ok(wrap_width) => spec.wrap.width = wrap_width,
            Err(error) => return Err(format!("{}", error)),
        }
    }
    spec.wrap.separator = utf8_decode("wrap_separator")?;
    if (spec.wrap.width == 0) ^ spec.wrap.separator.is_empty() {
        return Err("incomplete wrapping".to_string());
    }
    spec.translate.from = range::decode(&utf8_decode("translate_from")?)?;
    spec.translate.to = range::decode(&utf8_decode("translate_to")?)?;
    spec.encoding()
        .map(Some)
        .map_err(|error| format!("{}", error))
}

fn set_invalid_input(name: &str, id: i32) {
    let node = getElementById(&format!("{}_{}", name, id));
    addClass(&node, "invalid_input");
}

fn unset_invalid_input(name: &str, id: i32) {
    let node = getElementById(&format!("{}_{}", name, id));
    removeClass(&node, "invalid_input");
}

fn set_error(id: i32, message: &str) {
    let error = getElementById(&format!("error_{}", id));
    if message.is_empty() {
        removeClass(&error, "error");
    } else {
        addClass(&error, "error");
    }
    set_value(&error, message);
}

fn reset_errors() {
    for id in 0 .. next_id() {
        unset_invalid_input("text", id);
        unset_invalid_input("encoding", id);
        set_error(id, "");
    }
}

fn encoding_update(encoding: &Option<Encoding>, id: i32) {
    reset_errors();
    set_value(&getElementById(&format!("canonical_{}", id)), "");

    let spec = encoding
        .as_ref()
        .map(|e| e.specification())
        .unwrap_or_else(|| Specification::new());
    let set = |name, value: &str| {
        set_value(
            &getElementById(&format!("{}_{}", name, id)),
            &utf8::encode(value.as_bytes(), true),
        );
    };
    let set_range = |name, value| {
        set(name, &range::encode(value).unwrap());
    };

    set_range("symbols", &spec.symbols);
    let bit_order = match spec.bit_order {
        data_encoding::BitOrder::LeastSignificantFirst => "lsb",
        data_encoding::BitOrder::MostSignificantFirst => "msb",
    };
    set_checked(&getElementById(&format!("bit_order_{}_{}", bit_order, id)));
    if spec.check_trailing_bits {
        set_checked(&getElementById(&format!("trailing_bits_check_{}", id)));
    } else {
        set_checked(&getElementById(&format!("trailing_bits_ignore_{}", id)));
    }
    let mut padding = String::new();
    if let Some(c) = spec.padding {
        padding.push(c);
    }
    set("padding", &padding);
    set_range("ignore", &spec.ignore);
    if spec.wrap.width == 0 {
        set("wrap_width", "");
    } else {
        set("wrap_width", &format!("{}", spec.wrap.width));
    }
    set("wrap_separator", &spec.wrap.separator);
    set_range("translate_from", &spec.translate.from);
    set_range("translate_to", &spec.translate.to);

    if let Some(encoding) = encoding {
        let mut not = String::new();
        if !encoding.is_canonical() {
            not.push_str(" not");
        }
        set_value(
            &getElementById(&format!("canonical_{}", id)),
            if encoding.is_canonical() { "yes" } else { "no" },
        );
    }

    let input = value(&getElementById("input"));
    let output = match encoding {
        None => input,
        Some(encoding) => {
            let input = match utf8::decode(&input) {
                Ok(input) => input,
                Err(error) => {
                    set_error(id, &error);
                    return;
                }
            };
            utf8::encode(encoding.encode(&input).as_bytes(), false)
        }
    };
    set_value(&getElementById(&format!("text_{}", id)), &output);

    save_encoding(encoding, id);
}

fn swap_encoding(id: i32) {
    assert!(id > 0);
    let swap_value = |name: &str| {
        let left = getElementById(&format!("{}_{}", name, id - 1));
        let right = getElementById(&format!("{}_{}", name, id));
        let saved_value = value(&left);
        set_value(&left, &value(&right));
        set_value(&right, &saved_value);
    };
    let swap_checked = |name: &str, active: &str, default: &str| {
        let left = |side| getElementById(&format!("{}_{}_{}", name, side, id - 1));
        let right = |side| getElementById(&format!("{}_{}_{}", name, side, id));
        let saved_checked = is_checked(&left(active));
        if is_checked(&right(active)) {
            set_checked(&left(active));
        } else {
            set_checked(&left(default));
        }
        if saved_checked {
            set_checked(&right(active));
        } else {
            set_checked(&right(default));
        }
    };
    swap_value("text");
    swap_value("preset");
    swap_value("symbols");
    swap_checked("bit_order", "lsb", "msb");
    swap_checked("trailing_bits", "ignore", "check");
    swap_value("padding");
    swap_value("ignore");
    swap_value("wrap_width");
    swap_value("wrap_separator");
    swap_value("translate_from");
    swap_value("translate_to");
    swap_value("canonical");
    swap_value("error");
    spec_update(id - 1);
    spec_update(id);
}

fn read_state(name: &str) -> Option<String> {
    let value = getHistory(name);
    if value.is_null() {
        getStorage(name).as_string()
    } else {
        value.as_string()
    }
}

fn write_state(name: &str, value: &str) {
    setStorage(name, value);
    setHistory(name, value);
}

fn delete_state(name: &str) {
    deleteStorage(name);
    deleteHistory(name);
}

fn restore_encoding(id: i32, state: &[u8]) {
    let encoding = BASE64URL_NOPAD
        .decode(state)
        .ok()
        .and_then(|value| state::decode_encoding(&value));
    encoding_update(&encoding, id);
}

fn save_encoding(encoding: &Option<Encoding>, id: i32) {
    let name = format!("{}", id);
    match encoding {
        None => write_state(&name, ""),
        Some(encoding) => {
            let value = state::encode_encoding(&encoding);
            write_state(&name, &BASE64URL_NOPAD.encode(&value));
        }
    }
}

fn restore_input(state: &str) {
    let value = BASE64URL_NOPAD
        .decode(state.as_bytes())
        .ok()
        .and_then(|x| String::from_utf8(x).ok());
    set_value(
        &getElementById("input"),
        value.as_ref().map(String::as_str).unwrap_or(""),
    );
    save_input();
}

fn save_input() {
    write_state(
        "i",
        &BASE64URL_NOPAD.encode(value(&getElementById("input")).as_bytes()),
    );
}

const LIMIT_ID: i32 = 16;

fn next_id() -> i32 {
    for id in 0 .. LIMIT_ID {
        if getElementById(&format!("encoding_{}", id)).is_null() {
            return id;
        }
    }
    LIMIT_ID
}

#[wasm_bindgen]
pub fn init() {
    let encodings = createElement("div");
    setAttribute(&encodings, "id", "encodings");
    setAttribute(&encodings, "class", "encodings");
    appendChild(&body(), &encodings);

    let input = createElement("textarea");
    setAttribute(&input, "id", "input");
    setAttribute(&input, "style", "display: none;");
    appendChild(&encodings, &input);
    if let Some(state) = getHistory("i").as_string() {
        clearStorage();
        restore_input(&state);
    } else {
        restore_input(
            getStorage("i")
                .as_string()
                .as_ref()
                .map(String::as_str)
                .unwrap_or(""),
        );
    }

    for id in 0 .. LIMIT_ID {
        let state = match read_state(&format!("{}", id)) {
            None => break,
            Some(state) => state,
        };
        appendChild(&encodings, &create_encoding(id));
        restore_encoding(id, state.as_bytes());
        spec_update(id);
    }

    let next = createElement("div");
    setAttribute(&next, "id", "next");
    setAttribute(&next, "class", "encoding");
    let button = createElement("button");
    setAttribute(&button, "type", "button");
    setAttribute(&button, "onclick", "wasm_bindgen.add_encoding()");
    appendChild(&button, &createTextNode("+"));
    appendChild(&next, &button);
    appendChild(&encodings, &next);

    if next_id() > 0 {
        setAttribute(&getElementById("text_0"), "autofocus", "");
    }
}

#[wasm_bindgen]
pub fn swap_left(id: i32) {
    if id > 0 {
        swap_encoding(id);
    }
}

#[wasm_bindgen]
pub fn swap_right(id: i32) {
    if id < next_id() - 1 {
        swap_encoding(id + 1);
    }
}

#[wasm_bindgen]
pub fn delete_encoding(id: i32) {
    let next_id = next_id();
    for i in id + 1 .. next_id {
        swap_encoding(i);
    }
    removeChild(
        &getElementById("encodings"),
        &getElementById(&format!("encoding_{}", next_id - 1)),
    );
    delete_state(&format!("{}", next_id - 1));
}

#[wasm_bindgen]
pub fn add_encoding() {
    let id = next_id();
    if id == LIMIT_ID {
        return;
    }
    insertBefore(
        &getElementById("encodings"),
        &create_encoding(id),
        &getElementById("next"),
    );
    spec_update(id);
}

#[wasm_bindgen]
pub fn text_update(id: i32) {
    reset_errors();

    let mut input = match utf8::decode(&value(&getElementById(&format!("text_{}", id)))) {
        Ok(input) => input,
        Err(error) => {
            set_invalid_input("text", id);
            set_error(id, &error);
            return;
        }
    };

    match get_encoding(id) {
        Ok(None) => {}
        Ok(Some(encoding)) => match encoding.decode(&input) {
            Ok(result) => input = result,
            Err(error) => {
                set_invalid_input("text", id);
                set_error(id, &format!("{}", error));
                return;
            }
        },
        Err(error) => {
            set_invalid_input("encoding", id);
            set_error(id, &error);
            return;
        }
    }

    set_value(&getElementById("input"), &utf8::encode(&input, false));
    save_input();

    for i in 0 .. next_id() {
        if i == id {
            continue;
        }
        let output = getElementById(&format!("text_{}", i));
        match get_encoding(i) {
            Ok(None) => set_value(&output, &utf8::encode(&input, false)),
            Ok(Some(encoding)) => set_value(
                &output,
                &utf8::encode(encoding.encode(&input).as_bytes(), false),
            ),
            Err(error) => {
                set_invalid_input("encoding", i);
                set_error(i, &error);
            }
        }
    }
}

#[wasm_bindgen]
pub fn spec_update(id: i32) {
    reset_errors();
    set_value(&getElementById(&format!("canonical_{}", id)), "");
    set_value(&getElementById(&format!("preset_{}", id)), "");

    let encoding = match get_encoding(id) {
        Ok(encoding) => encoding,
        Err(error) => {
            set_invalid_input("encoding", id);
            set_error(id, &error);
            return;
        }
    };
    for (k, e) in PRESETS.iter() {
        if &encoding == e {
            set_value(&getElementById(&format!("preset_{}", id)), k);
        }
    }
    encoding_update(&encoding, id);
}

#[wasm_bindgen]
pub fn load_preset(id: i32) {
    reset_errors();

    let encoding = PRESETS
        .get(&value(&getElementById(&format!("preset_{}", id))))
        .unwrap();
    encoding_update(encoding, id);
}
