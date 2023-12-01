//* ret: 42
fn main() -> I64 {
    if (let x = true; x) {
        42
    } else {
        13
    }
}
