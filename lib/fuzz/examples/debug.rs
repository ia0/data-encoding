extern crate data_encoding_fuzz;

use std::io::Read;

use data_encoding_fuzz::generate_specification;

fn main() {
    let stdin = std::io::stdin();
    let mut input = Vec::new();
    stdin.lock().read_to_end(&mut input).unwrap();
    let mut data = &input[..];
    println!("{:#?}", generate_specification(&mut data));
    println!("spec = {:?}", &input[.. input.len() - data.len()]);
    println!("data = {:?}", data);
}
