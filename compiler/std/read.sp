fn read() -> I64 {
    let ASCII_NEWLINE = 10;
    let ASCII_DASH = 45;
    let ASCII_ZERO = 48;

    let mut next = read_char();
    let negative = next == ASCII_DASH;

    if negative {
        next = read_char();
    };

    let mut total = 0;

    // Exhaust all characters until a newline is hit.
    while next != ASCII_NEWLINE {
        total = total * 10 + next - ASCII_ZERO;
        next = read_char();
    };

    // If the number was negative, negate it.
    if negative {
        total = -total;
    };

    total
}

fn read_char() -> I64 {
    let mut v = 0;
    let mut res = 0;
    asm {
        subq $8 %RSP    // allocate stack space for reading char
        movq $0 %RAX    // read
        movq $0 %RDI    // stdin
        movq %RSP %RSI  // put read char at top of stack
        movq $1 %RDX    // read 1 byte
        syscall 4
        movq %RAX {res} // result of system call
        popq {v}        // pop read char
    };
    if res == 0 {
        return res
    };
    v
}