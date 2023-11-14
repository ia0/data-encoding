use data_encoding_fuzz::cmd;

fn main() {
    let path = cmd::path(true);
    let input = std::fs::read(&path).unwrap();
    cmd::execute(&cmd::target(&path), &input);
}
