//* ret: true
struct TestStruct {
    field_1: I64,
    field_2: Bool,
}

fn make_test() -> TestStruct {
    TestStruct { field_1: 5, field_2: true }
}

fn main() -> Bool {
    make_test().field_2
}
