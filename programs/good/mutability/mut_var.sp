//* ret: 42
fn test(mut x: I64) -> I64 {
    x = 42;
    x
}

fn main() -> I64 {
    test(8)
}
