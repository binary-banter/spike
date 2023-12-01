//* err: ConstructMissingField
struct TestStruct {
    field_1: I64,
    field_2: I64,
}

fn main() {
    let x = TestStruct {
        field_1: 13,
    };
}
