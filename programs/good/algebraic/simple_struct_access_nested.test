//* ret: 5
struct Foo {
    x : I64,
}

struct Bar {
    y: Foo,
}

fn main() -> I64 {
    let z = Bar { y: Foo { x : 5 } };
    z.y.x
}
