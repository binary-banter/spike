//* ret: 42
fn main() -> I64 {
    let x = (let x = 42; x);
    x
}
