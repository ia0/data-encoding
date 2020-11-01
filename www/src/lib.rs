extern crate data_encoding;
#[macro_use]
extern crate lazy_static;
extern crate wasm_bindgen;

mod range;
mod state;
mod utf8;

use data_encoding::{Encoding, Specification, BASE64URL_NOPAD};
use std::collections::HashMap;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    fn createElement(name: &str) -> JsValue;
    fn createTextNode(text: &str) -> JsValue;
    fn appendChild(parent: &JsValue, child: &JsValue);
    fn insertBefore(parent: &JsValue, child: &JsValue, node: &JsValue);
    fn removeChild(parent: &JsValue, child: &JsValue);
    fn setAttribute(node: &JsValue, name: &str, value: &str);
    fn removeAttribute(node: &JsValue, name: &str);
    fn getElementById(id: &str) -> JsValue;
    fn getElementByClass(node: &JsValue, name: &str) -> JsValue;
    fn value(node: &JsValue) -> String;
    fn set_value(node: &JsValue, value: &str);
    fn innerHTML(node: &JsValue) -> String;
    fn set_innerHTML(node: &JsValue, value: &str);
    fn focus(node: &JsValue);
    fn addClass(node: &JsValue, name: &str);
    fn removeClass(node: &JsValue, name: &str);
    fn hasClass(node: &JsValue, name: &str) -> bool;
    fn setStorage(name: &str, value: &str);
    fn getStorage(name: &str) -> JsValue;
    fn deleteStorage(name: &str);
    fn setHistory(name: &str, value: &str);
    fn getHistory(name: &str) -> JsValue;
    fn deleteHistory(name: &str);
}

const BUG_LINK: &'static str =
    "https://github.com/ia0/data-encoding/issues/new?labels=enhancement&title=[data-encoding.\
     rs]%20Short%20description%20of%20the%20bug&body=Steps%20to%20reproduce:\
     %0aExpected%20behavior:%0aActual%20behavior:";

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
    static ref TOP_BUTTONS: HashMap<String, Box<dyn Fn() -> JsValue + Sync>> = {
        let mut result: HashMap<String, Box<dyn Fn() -> JsValue + Sync>> = HashMap::new();
        result.insert("tutorial".to_string(), Box::new(|| create_tutorial()));
        result.insert("settings".to_string(), Box::new(|| create_settings()));
        result.insert("help".to_string(), Box::new(|| create_help()));
        result
    };
}

fn create_node(tag: &str, attributes: &[(&str, &str)], children: &[&JsValue]) -> JsValue {
    let node = createElement(tag);
    for &(name, value) in attributes {
        setAttribute(&node, name, value);
    }
    for child in children {
        appendChild(&node, child);
    }
    node
}

macro_rules! html {
    ({ $tag:ident [$($name:ident = $value:expr);*] $($children:tt)* }) => {
        create_node(stringify!($tag),
                    &[$((stringify!($name), $value)),*],
                    &[$(&html!($children)),*])
    };
    (($node:expr)) => { $node };
    ($text:expr) => { createTextNode($text) };
}

fn get_element(id: i32, name: &str) -> JsValue {
    getElementByClass(&getElementById(&format!("encoding_{}", id)), name)
}

fn with_id(fun: &str) -> String {
    format!("wasm_bindgen.{}(get_id(event))", fun)
}

fn ensure_class_if(node: &JsValue, class: &str, condition: bool) {
    if condition {
        addClass(node, class);
    } else {
        removeClass(node, class);
    }
}

fn ensure_enabled_if(node: &JsValue, enabled: bool) {
    if enabled {
        removeAttribute(node, "disabled");
        removeClass(node, "s_disabled");
    } else {
        setAttribute(node, "disabled", "");
        addClass(node, "s_disabled");
    }
}

fn create_tutorial() -> JsValue {
    let index = getStorage("tutorial").as_string();
    let index = match index {
        None => {
            return html!(
                "Using the `close tutorial` button in the top right of the page, close and reopen \
                 this tutorial page to advance to the next tutorial."
            )
        }
        Some(index) => index,
    };
    match index.as_str() {
        "init_done" => html! {
            { div []
              { p [] "Well done!" }
              { button [ type = "button";
                         onclick = "wasm_bindgen.goto_tutorial('intro')" ]
                "next tutorial" } }
        },
        "intro" => html!("TODO"),
        _ => panic!(),
    }
}

fn create_settings() -> JsValue {
    html! {
        { button [ type = "button"; onclick = "reset()" ] "reset" }
    }
}

fn create_help() -> JsValue {
    html!("There is no help yet.")
}

fn create_option(value: &str) -> JsValue {
    let node = createElement("option");
    setAttribute(&node, "value", value);
    appendChild(&node, &createTextNode(value));
    node
}

fn create_specification() -> JsValue {
    let spec_update = with_id("spec_update");

    let symbols_tooltip = "The number of symbols must be 2, 4, 8, 16, 32, or 64. Symbols must be \
                           ASCII characters (smaller than 128) and they must be unique.";
    let symbols = html! {
        { span []
          "Encode with "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "16";
                    placeholder = "symbols";
                    class = "i_symbols";
                    oninput = &spec_update;
                    title = symbols_tooltip ] }
        }
    };

    let bit_order_tooltip =
        "The default is to use most significant bit first since it is the most common.";
    let bit_order = html! {
        { span []
          { button [ type = "button";
                     class = "i_bit_order";
                     onclick = &with_id("toggle_bit_order");
                     title = bit_order_tooltip ]
            "Most"
          }
          " significant bit first"
        }
    };

    let trailing_bits_tooltip = "The default is to check trailing bits. This is ignored when \
                                 unnecessary (i.e. for base2, base4, and base16).";
    let trailing_bits = html! {
        { span []
          { button [ type = "button";
                     class = "i_trailing_bits";
                     onclick = &with_id("toggle_trailing_bits");
                     title = trailing_bits_tooltip ]
            "Check"
          }
          " trailing bits"
        }
    };

    let padding_tooltip = "The padding character must be ASCII and must not be a symbol.";
    let padding = html! {
        { span []
          "Pad with "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "6";
                    placeholder = "character";
                    class = "i_padding";
                    oninput = &spec_update;
                    title = padding_tooltip ] }
        }
    };

    let ignore_tooltip =
        "The characters to ignore must be ASCII and must not be symbols or the padding character.";
    let ignore = html! {
        { span []
          "Ignore "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "8";
                    placeholder = "characters";
                    class = "i_ignore";
                    oninput = &spec_update;
                    title = ignore_tooltip ] }
        }
    };

    let wrap_width_tooltip = "Must be a multiple of: 8 for base2, base8, and base32; 4 for base4 \
                              and base64; 2 for base16.";
    let wrap_separator_tooltip =
        "The wrapping characters must be ASCII and must not be symbols or the padding character.";
    let wrap = html! {
        { span []
          "Wrap every "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "4";
                    placeholder = "width";
                    class = "i_wrap_width";
                    oninput = &spec_update;
                    title = wrap_width_tooltip ] }
          " characters with "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "8";
                    placeholder = "separator";
                    class = "i_wrap_separator";
                    oninput = &spec_update;
                    title = wrap_separator_tooltip ] }
        }
    };

    let translate_from_tooltip = "The characters to translate from must be ASCII and must not \
                                  have already been assigned a semantics.";
    let translate_to_tooltip = "The characters to translate to must be ASCII and must have been \
                                assigned a semantics (symbol, padding character, or ignored \
                                character).";
    let translate = html! {
        { span []
          "Translate "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "12";
                    placeholder = "source";
                    class = "i_translate_from";
                    oninput = &spec_update;
                    title = translate_from_tooltip ] }
          " into "
          { input [ type = "text";
                    spellcheck = "false";
                    size = "12";
                    placeholder = "destination";
                    class = "i_translate_to";
                    oninput = &spec_update;
                    title = translate_to_tooltip ] }
        }
    };

    let canonical_tooltip = "The encoding is not canonical if trailing bits are not checked, \
                             padding is used, characters are ignored, or characters are \
                             translated.";
    let canonical = html! {
        { span []
          { output [ class = "i_canonical";
                     title = canonical_tooltip ] }
        }
    };

    html! {
        { div [ class = "s_specification" ]
          { div [ class = "s_control" ]
            { div [ class = "e_symbols" ] (symbols) }
            { div [ class = "e_bit_order" ] (bit_order) }
            { div [ class = "e_trailing_bits" ] (trailing_bits) }
            { div [ class = "e_padding" ] (padding) }
            { div [ class = "e_ignore" ] (ignore) }
            { div [ class = "e_wrap" ] (wrap) }
            { div [ class = "e_translate" ] (translate) } }
          { div [] (canonical) } }
    }
}

fn create_encoding(id: i32) -> JsValue {
    let preset = createElement("select");
    addClass(&preset, "s_preset");
    addClass(&preset, "i_preset");
    setAttribute(&preset, "onchange", &with_id("load_preset"));
    let mut presets: Vec<_> = PRESETS.keys().map(String::as_str).collect();
    presets.push("custom encoding");
    presets.sort();
    for x in presets {
        let option = create_option(x);
        if x == "custom encoding" {
            setAttribute(&option, "disabled", "");
        }
        appendChild(&preset, &option);
    }

    html! {
        { div [ id = &format!("encoding_{}", id);
                class = "i_encoding s_encoding" ]
          { div [ class = "s_menu" ]
            { button [ type = "button";
                       class = "i_swap_left";
                       onclick = &with_id("swap_left") ]
              "move left" }
            { button [ type = "button";
                       onclick = &with_id("delete_encoding") ]
              "delete" }
            { button [ type = "button";
                       class = "i_swap_right";
                       onclick = &with_id("swap_right") ]
              "move right" } }
          (preset)
          { textarea [ class = "i_text s_nofocus";
                       rows = "5";
                       cols = "50";
                       placeholder = "enter your text here";
                       oninput = &with_id("text_update");
                       onfocus = &with_id("text_update");
                       onkeydown = "text_keydown(event)" ] }
          (create_specification())
          { output [ class = "i_error" ] } }
    }
}

fn get_encoding(id: i32) -> Result<Option<Encoding>, String> {
    let utf8_decode = |name| -> Result<_, String> {
        let value = value(&get_element(id, name));
        Ok(String::from_utf8_lossy(&utf8::decode(&value)?).into_owned())
    };
    let symbols = utf8_decode("i_symbols")?;
    if symbols.is_empty() {
        return Ok(None);
    }
    let mut spec = Specification::new();
    spec.symbols = range::decode(&symbols)?;
    if innerHTML(&get_element(id, "i_bit_order")) == "Least" {
        spec.bit_order = data_encoding::BitOrder::LeastSignificantFirst;
    }
    if innerHTML(&get_element(id, "i_trailing_bits")) == "Ignore" {
        spec.check_trailing_bits = false;
    }
    let padding = utf8_decode("i_padding")?;
    let mut padding_iter = padding.chars().fuse();
    spec.padding = padding_iter.next();
    if padding_iter.next().is_some() {
        return Err("padding has more than one character".to_string());
    }
    match spec.symbols.len() {
        2 | 4 | 16 => spec.padding = None,
        _ => (),
    }
    spec.ignore = range::decode(&utf8_decode("i_ignore")?)?;
    let wrap_width = value(&get_element(id, "i_wrap_width"));
    if !wrap_width.is_empty() {
        match wrap_width.parse() {
            Ok(wrap_width) => spec.wrap.width = wrap_width,
            Err(error) => return Err(format!("{}", error)),
        }
    }
    spec.wrap.separator = utf8_decode("i_wrap_separator")?;
    if (spec.wrap.width == 0) ^ spec.wrap.separator.is_empty() {
        return Err("incomplete wrapping".to_string());
    }
    spec.translate.from = range::decode(&utf8_decode("i_translate_from")?)?;
    spec.translate.to = range::decode(&utf8_decode("i_translate_to")?)?;
    spec.encoding().map(Some).map_err(|error| format!("{}", error))
}

fn set_invalid_input(name: &str, id: i32) {
    addClass(&get_element(id, name), "s_invalid_input");
}

fn unset_invalid_input(name: &str, id: i32) {
    removeClass(&get_element(id, name), "s_invalid_input");
}

fn set_error(id: i32, message: &str) {
    set_value(&get_element(id, "i_error"), message);
}

fn reset_errors() {
    for id in 0 .. next_id() {
        unset_invalid_input("i_text", id);
        unset_invalid_input("i_encoding", id);
        set_error(id, "");
    }
}

fn fix_swap_buttons() {
    let last_id = next_id() - 1;
    for id in 0 ..= last_id {
        ensure_enabled_if(&get_element(id, "i_swap_left"), id != 0);
        ensure_enabled_if(&get_element(id, "i_swap_right"), id != last_id);
    }
}

fn encoding_update(encoding: &Option<Encoding>, id: i32) {
    reset_errors();
    set_value(&get_element(id, "i_canonical"), "");

    let spec = encoding.as_ref().map(|e| e.specification()).unwrap_or_else(|| Specification::new());
    #[derive(PartialEq, Eq)]
    enum Hide {
        AllButSymbol,
        TrailingBits,
        Nothing,
    };
    let hide = match encoding {
        None => Hide::AllButSymbol,
        Some(encoding) => match encoding.bit_width() {
            1 | 2 | 4 => Hide::TrailingBits,
            _ => Hide::Nothing,
        },
    };
    let set = |name, value: &str| {
        set_value(&get_element(id, name), &utf8::encode(value.as_bytes(), true));
    };
    let set_range = |name, value| {
        set(name, &range::encode(value).unwrap());
    };
    let ensure = |class, name, condition| {
        ensure_class_if(&get_element(id, name), class, condition);
    };

    set_range("i_symbols", &spec.symbols);
    ensure("s_disabled", "e_symbols", spec.symbols.is_empty());
    let bit_order = match spec.bit_order {
        data_encoding::BitOrder::LeastSignificantFirst => "Least",
        data_encoding::BitOrder::MostSignificantFirst => "Most",
    };
    set_innerHTML(&get_element(id, "i_bit_order"), bit_order);
    ensure("s_hidden", "e_bit_order", hide == Hide::AllButSymbol);
    let trailing_bits = match spec.check_trailing_bits {
        true => "Check",
        false => "Ignore",
    };
    set_innerHTML(&get_element(id, "i_trailing_bits"), trailing_bits);
    ensure("s_hidden", "e_trailing_bits", hide != Hide::Nothing);
    let mut padding = String::new();
    if let Some(c) = spec.padding {
        padding.push(c);
    }
    set("i_padding", &padding);
    ensure("s_disabled", "e_padding", padding.is_empty());
    ensure("s_hidden", "e_padding", hide != Hide::Nothing);
    set_range("i_ignore", &spec.ignore);
    ensure("s_disabled", "e_ignore", spec.ignore.is_empty());
    ensure("s_hidden", "e_ignore", hide == Hide::AllButSymbol);
    if spec.wrap.width == 0 {
        set("i_wrap_width", "");
    } else {
        set("i_wrap_width", &format!("{}", spec.wrap.width));
    }
    set("i_wrap_separator", &spec.wrap.separator);
    ensure("s_disabled", "e_wrap", spec.wrap.width == 0);
    ensure("s_hidden", "e_wrap", hide == Hide::AllButSymbol);
    set_range("i_translate_from", &spec.translate.from);
    set_range("i_translate_to", &spec.translate.to);
    ensure("s_disabled", "e_translate", spec.translate.from.is_empty());
    ensure("s_hidden", "e_translate", hide == Hide::AllButSymbol);

    if let Some(encoding) = encoding {
        let mut canonical = "Encoding is".to_string();
        if !encoding.is_canonical() {
            canonical.push_str(" not");
        }
        canonical.push_str(" canonical");
        set_value(&get_element(id, "i_canonical"), &canonical);
    }

    let text = get_element(id, "i_text");
    if hasClass(&text, "s_nofocus") {
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
        setAttribute(&text, "spellcheck", &format!("{}", encoding.is_none()));
        set_value(&text, &output);
    } else {
        text_update(id);
    }

    save_encoding(encoding, id);
}

fn swap_encoding(id: i32) {
    assert!(id > 0);
    let prev = get_element(id - 1, "i_encoding");
    let next = get_element(id, "i_encoding");
    insertBefore(&getElementById("encodings"), &next, &prev);
    setAttribute(&prev, "id", &format!("encoding_{}", id));
    setAttribute(&next, "id", &format!("encoding_{}", id - 1));
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
    let encoding =
        BASE64URL_NOPAD.decode(state).ok().and_then(|value| state::decode_encoding(&value));
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
    let value =
        BASE64URL_NOPAD.decode(state.as_bytes()).ok().and_then(|x| String::from_utf8(x).ok());
    let input = value.as_ref().map(String::as_str).unwrap_or("");
    set_value(&getElementById("input"), input);
    save_input(input);
}

fn save_input(input: &str) {
    write_state("i", &BASE64URL_NOPAD.encode(input.as_bytes()));
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
    let top_buttons = html! {
        { div [ class = "s_top_right" ]
          { a [ href = BUG_LINK; target = "_blank" ]
            { button [ type = "button" ] "report bug" } } }
    };
    for name in ["tutorial", "settings", "help"].iter() {
        let button = html! {
            { button [ type = "button";
                       id = &format!("toggle_{}", name);
                       onclick = &format!("wasm_bindgen.toggle_menu('{}')", name) ] }
        };
        appendChild(&button, &createTextNode(&format!("open {}", name)));
        appendChild(&top_buttons, &button);
    }

    let top = html! {
        { div [ id = "top"; class = "s_top" ]
          { div [ class = "s_top_menu" ]
            { div [ id = "title"; class = "s_title" ] }
            (top_buttons) } }
    };
    appendChild(&getElementById("everything"), &top);

    if getStorage("tutorial").is_null() {
        toggle_menu("tutorial");
    }

    let encodings = html! {
        { div [ id = "encodings"; class = "s_encodings" ] }
    };
    appendChild(&getElementById("everything"), &encodings);

    let input = createElement("textarea");
    setAttribute(&input, "id", "input");
    setAttribute(&input, "style", "display: none;");
    appendChild(&encodings, &input);
    if let Some(state) = getHistory("i").as_string() {
        for id in 0 .. LIMIT_ID {
            let name = format!("{}", id);
            if getStorage(&name).is_null() {
                break;
            }
            deleteStorage(&name);
        }
        restore_input(&state);
    } else {
        if let Some(state) = getStorage("i").as_string() {
            restore_input(&state);
        } else {
            restore_input(&BASE64URL_NOPAD.encode(b"hello"));
            save_encoding(&None, 0);
            save_encoding(&Some(data_encoding::BASE64), 1);
        }
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
    fix_swap_buttons();

    let next = html! {
        { div [ id = "next"; class = "s_next s_encoding" ]
          { button [ type = "button";
                     onclick = "wasm_bindgen.add_encoding()" ]
            "add" } }
    };
    appendChild(&encodings, &next);

    if next_id() > 0 {
        let text = get_element(0, "i_text");
        setAttribute(&text, "autofocus", "");
        removeClass(&text, "s_nofocus");
    }

    let nothing = getElementById("nothing");
    set_innerHTML(&nothing, "");
    setAttribute(&nothing, "style", "display: none;");
    removeAttribute(&getElementById("everything"), "style");
}

#[wasm_bindgen]
pub fn swap_left(id: i32) {
    if id > 0 {
        swap_encoding(id);
    }
    fix_swap_buttons();
}

#[wasm_bindgen]
pub fn swap_right(id: i32) {
    if id < next_id() - 1 {
        swap_encoding(id + 1);
    }
    fix_swap_buttons();
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
    fix_swap_buttons();
    delete_state(&format!("{}", next_id - 1));
}

#[wasm_bindgen]
pub fn add_encoding() {
    let id = next_id();
    if id == LIMIT_ID {
        return;
    }
    insertBefore(&getElementById("encodings"), &create_encoding(id), &getElementById("next"));
    fix_swap_buttons();
    spec_update(id);
}

#[wasm_bindgen]
pub fn toggle_bit_order(id: i32) {
    let toggle = get_element(id, "i_bit_order");
    if innerHTML(&toggle) != "Least" {
        set_innerHTML(&toggle, "Least");
    } else {
        set_innerHTML(&toggle, "Most");
    }
    spec_update(id);
}

#[wasm_bindgen]
pub fn toggle_trailing_bits(id: i32) {
    let toggle = get_element(id, "i_trailing_bits");
    if innerHTML(&toggle) != "Ignore" {
        set_innerHTML(&toggle, "Ignore");
    } else {
        set_innerHTML(&toggle, "Check");
    }
    spec_update(id);
}

#[wasm_bindgen]
pub fn text_update(id: i32) {
    reset_errors();
    for i in 0 .. next_id() {
        ensure_class_if(&get_element(i, "i_text"), "s_nofocus", i != id);
    }
    removeClass(&get_element(id, "i_text"), "s_nofocus");

    let mut input = match utf8::decode(&value(&get_element(id, "i_text"))) {
        Ok(input) => input,
        Err(error) => {
            set_invalid_input("i_text", id);
            set_error(id, &error);
            return;
        }
    };

    match get_encoding(id) {
        Ok(None) => {}
        Ok(Some(encoding)) => match encoding.decode(&input) {
            Ok(result) => input = result,
            Err(error) => {
                set_invalid_input("i_text", id);
                set_error(id, &format!("{}", error));
                return;
            }
        },
        Err(error) => {
            set_invalid_input("i_encoding", id);
            set_error(id, &error);
            return;
        }
    }

    let encoded_input = utf8::encode(&input, false);
    set_value(&getElementById("input"), &encoded_input);
    save_input(&encoded_input);

    for i in 0 .. next_id() {
        if i == id {
            continue;
        }
        let output = get_element(i, "i_text");
        match get_encoding(i) {
            Ok(None) => set_value(&output, &encoded_input),
            Ok(Some(encoding)) => {
                set_value(&output, &utf8::encode(encoding.encode(&input).as_bytes(), false))
            }
            Err(error) => {
                set_invalid_input("i_encoding", i);
                set_error(i, &error);
            }
        }
    }
}

#[wasm_bindgen]
pub fn spec_update(id: i32) {
    reset_errors();
    set_value(&get_element(id, "i_preset"), "custom encoding");
    set_value(&get_element(id, "i_canonical"), "");

    let encoding = match get_encoding(id) {
        Ok(encoding) => encoding,
        Err(error) => {
            set_invalid_input("i_encoding", id);
            set_error(id, &error);
            return;
        }
    };
    for (k, e) in PRESETS.iter() {
        if &encoding == e {
            set_value(&get_element(id, "i_preset"), k);
        }
    }
    encoding_update(&encoding, id);
}

#[wasm_bindgen]
pub fn load_preset(id: i32) {
    let encoding = PRESETS.get(&value(&get_element(id, "i_preset"))).unwrap();
    encoding_update(encoding, id);
}

#[wasm_bindgen]
pub fn toggle_menu(name: &str) {
    let content = match TOP_BUTTONS.get(name) {
        None => return,
        Some(content) => content,
    };
    let top = getElementById("top");
    let title = getElementById("title");
    let old_title = innerHTML(&title);
    if old_title != "" {
        set_innerHTML(&title, "");
        removeChild(&top, &getElementById("content"));
        set_innerHTML(
            &getElementById(&format!("toggle_{}", &old_title)),
            &format!("open {}", old_title),
        );
        if old_title == name {
            if name == "tutorial" && getStorage("tutorial").is_null() {
                setStorage("tutorial", "init_done");
            }
            return;
        }
    }
    set_innerHTML(&title, name);
    let new_content = html! {
        { div [ id = "content"; class = "s_content" ] (content()) }
    };
    appendChild(&top, &new_content);
    set_innerHTML(&getElementById(&format!("toggle_{}", name)), &format!("close {}", name));
}

#[wasm_bindgen]
pub fn goto_tutorial(name: &str) {
    setStorage("tutorial", name);
    // TODO: Make this better.
    toggle_menu("tutorial");
    toggle_menu("tutorial");
}

#[wasm_bindgen]
pub fn move_focus(id: i32) {
    let id = if id < 0 { next_id() - 1 } else { id % next_id() };
    focus(&get_element(id, "i_text"));
}
