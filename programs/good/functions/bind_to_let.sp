//* ret: 42
fn add(x: I64, y: I64) -> I64 {
    x + y
}

fn main() -> I64 {
    let add2 = add;
    add2(40, 2)
}
