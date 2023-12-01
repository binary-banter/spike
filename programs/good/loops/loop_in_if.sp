//* ret: 5
fn main() -> I64 {
    if (loop { break true; }) {
        5
    } else {
        3
    }
}
