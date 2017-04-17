extern crate gcc;

fn main() {
    for compiler in ["clang", "gcc"].iter() {
        gcc::Config::new()
            .compiler(compiler)
            .define("COMPILER", Some(compiler))
            .file("src/ref.c")
            .compile(&format!("libref_{}.a", compiler));
    }
}
