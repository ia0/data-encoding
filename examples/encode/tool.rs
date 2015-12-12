macro_rules! check {
    ($c: expr, $e: expr) => {
        if !$c {
            return Err($e);
        }
    };
}
