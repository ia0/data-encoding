extern crate cc;

fn main() {
    for compiler in ["clang", "gcc"].iter() {
        cc::Build::new()
            .compiler(compiler)
            .define("COMPILER", Some(*compiler))
            .file("src/ref.c")
            .flag_if_supported("-Wno-unterminated-string-initialization")
            .compile(&format!("libref_{}.a", compiler));
    }
}
