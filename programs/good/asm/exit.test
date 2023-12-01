//* out: 0
//* ret: 42
fn exit_asm(exit_code: I64) {
    asm {
        movq {exit_code} %RDI
        movq $60 %RAX
        syscall 2
    };
}

fn main() {
    print(0);
    exit_asm(42);
    print(1);
}
