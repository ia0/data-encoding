use std::collections::{BTreeMap, HashMap, HashSet};

use data_encoding_fuzz::cmd;

fn main() {
    let path = cmd::path(false);
    let target = cmd::target(&path);
    let mut stats = Stats::new(std::env::args().skip(2));
    for entry in std::fs::read_dir(path).unwrap() {
        stats.merge(&cmd::execute(&target, &std::fs::read(entry.unwrap().path()).unwrap()));
    }
    stats.print();
}

struct Stats {
    buckets: Vec<(String, Bucket)>,
    filters: Vec<(String, Filter)>,
    stats: HashMap<Vec<Option<usize>>, HashMap<&'static str, Stat>>,
}

#[derive(Clone, Copy, Default)]
struct Stat {
    sum: f64,
    len: usize,
}

#[derive(Clone, Copy)]
enum Bucket {
    Lin(usize),
    Exp(usize),
}

#[derive(Clone, Copy)]
enum Filter {
    Is(usize),
    Eq(usize),
    Lt(usize),
    Gt(usize),
}

impl Stats {
    fn new(args: impl Iterator<Item = String>) -> Self {
        let mut buckets = BTreeMap::new();
        let mut filters = Vec::new();
        for arg in args {
            let Some((name, value)) = arg.split_once(['+', '*', '!', '=', '<', '>']) else {
                panic!("{arg:?} does not contain an operator: + * ! = < >");
            };
            if !name.bytes().all(|x| b"abcdefghijklmnopqrstuvwxz_".contains(&x)) {
                panic!("{name:?} is not a name");
            }
            let Ok(value) = value.parse::<usize>() else {
                panic!("{value:?} is not a value");
            };
            let op = match arg.as_bytes()[name.len()] {
                b'+' => Ok(Bucket::Lin(value)),
                b'*' => Ok(Bucket::Exp(value)),
                b'!' => Err(Filter::Is(value)),
                b'=' => Err(Filter::Eq(value)),
                b'<' => Err(Filter::Lt(value)),
                b'>' => Err(Filter::Gt(value)),
                _ => unreachable!(),
            };
            match op {
                Ok(bucket) => {
                    if buckets.insert(name.to_string(), bucket).is_some() {
                        panic!("duplicate bucket for {name}");
                    }
                }
                Err(filter) => filters.push((name.to_string(), filter)),
            }
        }
        let buckets: Vec<_> = buckets.into_iter().collect();
        Stats { buckets, filters, stats: HashMap::new() }
    }

    fn merge(&mut self, stat: &HashMap<&'static str, usize>) {
        let Stats { buckets, filters, stats } = self;
        let slot = buckets.iter().map(|(k, b)| stat.get(k.as_str()).map(|&v| b.slot(v))).collect();
        if !filters.iter().all(|x| x.1.contains(stat.get(x.0.as_str()).copied())) {
            return;
        }
        let stats = stats.entry(slot).or_default();
        for (&key, &value) in stat {
            stats.entry(key).or_default().merge(value);
        }
        stats.entry(COUNT).or_default().merge(1);
    }

    fn print(self) {
        // Compute slot headers.
        let slot_hdrs: Vec<_> = self.buckets.iter().map(|x| x.0.as_str()).collect();
        assert!(slot_hdrs.is_sorted());

        // Compute stat headers.
        let mut stat_hdrs = HashSet::new();
        for stats in self.stats.values() {
            stat_hdrs.extend(stats.keys().copied());
        }
        let mut stat_hdrs: Vec<_> = stat_hdrs.into_iter().collect();
        stat_hdrs.sort();
        let stat_hdrs = stat_hdrs;

        // Compute columns (a column is a slot and its stat).
        let mut cols = Vec::new();
        for (k, v) in self.stats {
            let mut t = Vec::new();
            for &x in &stat_hdrs {
                t.push(v.get(x).copied());
            }
            cols.push((k, t));
        }
        cols.sort_by(|x, y| x.0.cmp(&y.0));

        // Compute matrix.
        let mut matrix = vec![Vec::new(); slot_hdrs.len() + stat_hdrs.len()];
        let n = slot_hdrs.len();
        for (i, h) in slot_hdrs.iter().enumerate() {
            matrix[i].push(h.to_string());
        }
        for (i, h) in stat_hdrs.iter().enumerate() {
            matrix[n + i].push(h.to_string());
        }
        for (slot, stat) in cols {
            for (i, x) in slot.into_iter().enumerate() {
                matrix[i].push(x.map_or("-".to_string(), |x| format!("{x}..")));
            }
            for (i, x) in stat.into_iter().enumerate() {
                let cell = match x {
                    Some(x) if stat_hdrs[i] == COUNT => x.len.to_string(),
                    Some(x) => format!("{:.2}", x.average()),
                    None => "-".to_string(),
                };
                matrix[n + i].push(cell);
            }
        }

        // Print matrix.
        print_matrix(matrix);
    }
}

impl Stat {
    fn merge(&mut self, value: usize) {
        self.sum += value as f64;
        self.len += 1;
    }

    fn average(self) -> f64 {
        self.sum / self.len as f64
    }
}

impl Bucket {
    fn slot(self, mut value: usize) -> usize {
        match self {
            Bucket::Lin(delta) => value / delta * delta,
            Bucket::Exp(base) => {
                let mut slot = 0;
                while 0 < value {
                    value /= base;
                    slot = if slot == 0 { 1 } else { slot * base };
                }
                slot
            }
        }
    }
}

impl Filter {
    fn contains(self, value: Option<usize>) -> bool {
        match self {
            Filter::Is(x) => value.is_some() as usize == x,
            Filter::Eq(x) => value.is_some_and(|y| y == x),
            Filter::Lt(x) => value.is_some_and(|y| y < x),
            Filter::Gt(x) => value.is_some_and(|y| y > x),
        }
    }
}

const COUNT: &str = "-- count --";

fn align(x: &str, n: usize) {
    for _ in 0 .. n.saturating_sub(x.len()) {
        print!(" ");
    }
    print!("{x}")
}

fn print_matrix(mut m: Vec<Vec<String>>) {
    let Some(n) = m.iter().map(|r| r.len()).max() else { return };
    m.iter_mut().for_each(|x| x.resize(n, String::new()));
    let w: Vec<_> =
        (0 .. n).map(|i| m.iter().map(|x| x[i].len()).max().unwrap() + (i != 0) as usize).collect();
    for x in m {
        for i in 0 .. n {
            align(&x[i], w[i]);
        }
        println!();
    }
}
