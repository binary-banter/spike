fn print(mut v: I64) -> I64 {
    if v < 0 {
        print_char(b'-');
        _print_rec(-v); // todo: this is not correct when v assumes its maximum negative value
    } else if v == 0 {
        print_char(b'0');
    } else {
        _print_rec(v);
    }
    print_char(b'\n');
    v
}

fn _print_rec(v: I64) {
    if v == 0 {
        return;
    }
    _print_rec(v / 10);
    print_char((v % 10) + b'0');
}

fn print_char(v: I64) {
    asm {
        pushq {v}
        movq $1 %RAX    // write
        movq $1 %RDI    // stdout
        movq %RSP %RSI  // print char on top of stack
        movq $1 %RDX    // print 1 byte
        syscall 4
        addq $8 %RSP    // reset stack
    };
}
