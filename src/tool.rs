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

pub unsafe fn chunk_unchecked(x: &[u8], n: usize, i: usize) -> &[u8] {
    let ptr = x.as_ptr().offset((n * i) as isize);
    ::std::slice::from_raw_parts(ptr, n)
}

pub unsafe fn chunk_mut_unchecked
    (x: &mut [u8], n: usize, i: usize) -> &mut [u8]
{
    let ptr = x.as_mut_ptr().offset((n * i) as isize);
    ::std::slice::from_raw_parts_mut(ptr, n)
}

pub fn chunk(x: &[u8], n: usize, i: usize) -> &[u8] {
    assert!(n * (i + 1) <= x.len());
    unsafe { chunk_unchecked(x, n, i) }
}

pub fn chunk_mut(x: &mut [u8], n: usize, i: usize) -> &mut [u8] {
    assert!(n * (i + 1) <= x.len());
    unsafe { chunk_mut_unchecked(x, n, i) }
}
