//* ret: 34
fn fib(n: I64) -> I64 {
    if n <= 1 {
        if n  == 0 { 0 } else { 1 }
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
fn main() -> I64 {
    fib(9)
}
