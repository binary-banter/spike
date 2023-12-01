//* ret: 5
struct Foo {
    a : I64,
    b : Bool
}

struct Bar {
    c: Foo,
    d: I64
}

fn test(x: Bar, y: Bool) -> I64 {
    5
}

fn main() -> I64 {
    let z = Bar { c: Foo { a : 5, b: true }, d: 7 };
    test(z, false)
}
