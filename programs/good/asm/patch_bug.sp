// This test allocates every register and performs an instruction that will result in two operands being dereferences.
//* out: 2 3 4 5 6 7 8 9 10 11 12 13 14 15
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
    };
    asm { movq %RAX {res} };
    print(res);
    asm { movq %RBX {res} };
    print(res);
    asm { movq %RCX {res} };
    print(res);
    asm { movq %RDX {res} };
    print(res);
    asm { movq %RSI {res} };
    print(res);
    asm { movq %RDI {res} };
    print(res);
    asm { movq %R8 {res} };
    print(res);
    asm { movq %R9 {res} };
    print(res);
    asm { movq %R10 {res} };
    print(res);
    asm { movq %R11 {res} };
    print(res);
    asm { movq %R12 {res} };
    print(res);
    asm { movq %R13 {res} };
    print(res);
    asm { movq %R14 {res} };
    print(res);
    asm { movq %R15 {res} };
    print(res);
}
