//* out: 76 38 19 58 29 88 44 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1
//* ret: 0
fn div(n: I64) -> I64 {
    n / 2
}

fn mul(n: I64) -> I64 {
    1 + 3 * n
}

fn col(n: I64) -> Bool {
    if n == 1 {
        true
    } else {
        let fun = (if n % 2 == 0 { div } else { mul });
        col(print(fun(n)))
    }
}

fn main() -> I64 {
    if col(25) {
        0
    } else {
        1
    }
}
