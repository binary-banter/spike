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

fn max(lhs: I64, rhs: I64) -> I64 {
    if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

fn main() {
    let mut total = 0;
    let mut next = 0;

    while (next = read_char(); next != b'\0') {
        while (next = read_char(); next != b':') { unit };

        let mut red = 0;
        let mut green = 0;
        let mut blue = 0;

        while next != b'\n' {
            next = read_char(); // space

            let mut number = 0;
            while (next = read_char(); next >= b'0' && next <= b'9') {
                number = number * 10 + next - b'0';
            };

            next = read_char(); // r | g | b

            if next == b'r' {
                red = max(red, number);
                next = read_char(); // e
                next = read_char(); // d
            } else if next == b'g' {
                green = max(green, number);
                next = read_char(); // r
                next = read_char(); // e
                next = read_char(); // e
                next = read_char(); // n
            } else if next == b'b' {
                blue = max(blue, number);
                next = read_char(); // l
                next = read_char(); // u
                next = read_char(); // e
            } else {
                exit(1)
            };

            next = read_char(); // newline | comma | semicolon
        };

        total = total + red * green * blue;
    };
    print(total);
}
