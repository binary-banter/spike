//* inp: 1 2 3 4 5 6 7 8 9 10 0
//* ret: 55
fn main() -> I64 {
    let mut x = 0;
    let mut sum = 0;
    while (x = read(); x != 0) {
        sum = sum + x
    }
    sum
}
