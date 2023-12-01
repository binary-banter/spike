//* inp: 1 1 2 2 3 3
//* ret: 42
fn main() -> I64 {
    let x1 = read();
    let x2 = read();
    let x3 = read();
    let x4 = read();
    let x5 = read();
    let x6 = read();
    (x1 - x2) + (x3 - x4) + (x5 - x6) + 42
}
