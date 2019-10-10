use std::collections::HashMap;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let mut stats: HashMap<String, usize> = HashMap::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line: Vec<_> = line.splitn(2, ' ').collect();
        let count: usize = line[0].parse().unwrap();
        let key = line[1];
        *stats.entry(key.into()).or_default() += count;
    }
    for (key, count) in stats {
        println!("{} {}", count, key);
    }
}
