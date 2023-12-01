//* ret: 42
fn add(x: I64, y: I64) -> I64 {
    x + y
}

fn mul(x: I64, y: I64) -> I64 {
    x * y
}

fn main() -> I64 {
    mul(add(2, 4), 7)
}
