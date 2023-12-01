//* ret: 42
fn main() -> I64 {
    let mut x = 1;
    loop {
        if x < 42 {
            x = x + 1;
        } else {
            break x;
        }
    }
}
