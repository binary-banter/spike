struct Buffer {
    ptr: I64,
    len: I64,
}

// fn get(buffer: Buffer, index: I64) -> I64 {
//
// }

fn read_all_to_buffer() -> Buffer {
    let start = malloc(0x10_00_00);
    print(start);
    loop { unit };

    let mut ptr = start;
    let mut res = 0i64;

    loop {
        asm{
            movq $0 %RAX      // read
            movq $0 %RDI      // stdin
            movq {ptr} %RSI
            movq $1 %RDX      // read 1 byte
            syscall 4
            movq %RAX {res}   // result of system call
        };
        print(ptr);
        if res == b'\0' {
            break;
        };
        ptr = ptr + 8;
    };

    let len = (ptr - start) / 8;

    Buffer {
        ptr: start,
        len,
    }
}

fn main() {
    read_all_to_buffer();
}
