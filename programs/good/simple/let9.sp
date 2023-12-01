//* inp: 7 9
//* ret: 16
fn main() -> I64 {
    let x = read();
    x + (let x = read(); x)
}
