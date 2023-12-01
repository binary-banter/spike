//* out: 42
fn print_backwards(mut v: I64) {
    let ASCII_ZERO = 48;
    while v != 0 {
        let unit_digit = v % 10;
        print_char(unit_digit + ASCII_ZERO);
        v = v / 10;
    };
    let ASCII_NEWLINE = 10;
    print_char(ASCII_NEWLINE);
}

fn print_char(v: I64) {
    asm {
        pushq {v}
        movq $1 %RAX    // write
        movq $1 %RDI    // stdout
        movq %RSP %RSI  // print char on top of stack
        movq $1 %RDX    // print 1 byte
        syscall 4       // arity of 4
        addq $8 %RSP    // reset stack
    };
}

fn main() {
    print_backwards(24);
}
