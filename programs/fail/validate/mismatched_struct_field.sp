//* err: MismatchedStructField
struct TestStruct {
    field_1: I64,
}

fn main() {
    TestStruct {
        field_1: true,
    };
}
