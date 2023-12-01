//* err: UnsizedType
struct TestStruct1 {
    field_1: TestStruct2,
}

struct TestStruct2 {
    field_1: TestStruct1,
}

fn main() {}
