//* ret: 8
fn main() -> I64 {
    let mut x = 1;
    let y = 4i64;
    asm {
        addq $1 {x}
        movq {x} %RAX
        mulq {y}
        movq %RAX {x}
    }
    x
}
