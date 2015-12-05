macro_rules! check {
    ($e: expr, $c: expr) => {
        if !$c {
            return Err($e);
        }
    };
}

pub fn div_ceil(x: usize, m: usize) -> usize {
    (x + m - 1) / m
}

pub fn chunk(x: &[u8], n: usize, i: usize) -> &[u8] {
    &x[n * i .. n * (i + 1)]
}

pub fn chunk_mut(x: &mut [u8], n: usize, i: usize) -> &mut [u8] {
    &mut x[n * i .. n * (i + 1)]
}
