fn read_char() -> I64 {
    let mut v = 0;
    let mut res = 0;
    asm {
        subq $8 %RSP    // allocate stack space for reading char
        movq $0 %RAX    // read
        movq $0 %RDI    // stdin
        movq %RSP %RSI  // put read char at top of stack
        movq $1 %RDX    // read 1 byte
        syscall 4       // arity of 4
        movq %RAX {res} // result of system call
        popq {v}        // pop read char
    };
    if res == 0 {
        return res
    };
    v
}

fn main() {
    let ASCII_NEWLINE = 10;
    let ASCII_ZERO = 48;
    let ASCII_NULL = 0;

    let mut next = read_char();

    let mut best = 0;
    let mut sum = 0;
    let mut current = 0;
    let mut last_was_newline = false;

    while next != ASCII_NULL {
        if next == ASCII_NEWLINE {
            if last_was_newline {
                // Found empty line
                if sum > best {
                    best = sum;
                };
                sum = 0;
            } else {
                last_was_newline = true;
                sum = sum + current;
                current = 0;
            }
        } else {
            last_was_newline = false;
            current = current * 10 + next - ASCII_ZERO;
        };
        next = read_char();
    };
    print(best);
}
