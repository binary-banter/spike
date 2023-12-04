//* ret: 5
fn main() -> I64 {
    loop {
        let x = (loop { break true });
        if x {
            break 5
        } else {
            break 7
        }
    }
}
