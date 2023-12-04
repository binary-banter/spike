// This test allocates every register and performs an instruction that will result in two operands being dereferences.
//* out: 15 14 13 12 11 10 9 8 7 6 5 4 3 2
fn main() {
    let mut res = 0i64;
    asm {
        movq $2 %RAX
        movq $3 %RBX
        movq $4 %RCX
        movq $5 %RDX
        movq $6 %RSI
        movq $7 %RDI
        movq $8 %R8
        movq $9 %R9
        movq $10 %R10
        movq $11 %R11
        movq $12 %R12
        movq $13 %R13
        movq $14 %R14
        movq $15 %R15
        movq {res} {res}
        pushq %RAX
        pushq %RBX
        pushq %RCX
        pushq %RDX
        pushq %RSI
        pushq %RDI
        pushq %R8
        pushq %R9
        pushq %R10
        pushq %R11
        pushq %R12
        pushq %R13
        pushq %R14
        pushq %R15
    }
    let mut i = 14i64;
    while (i = i - 1; i >= 0) {
        asm { popq {res} };
        print(res);
    };
}
