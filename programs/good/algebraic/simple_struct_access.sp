//* ret: 5
struct TestStruct {
    field_1: I64,
    field_2: Bool,
}

fn main() -> I64 {
    let x = TestStruct {
        field_1: 5,
        field_2: true,
    };
    x.field_1
}
