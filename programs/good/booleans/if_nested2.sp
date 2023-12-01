//* ret: 2
fn main() -> I64 {
    if (if false { true } else { false }) { 1 } else { 2 }
}