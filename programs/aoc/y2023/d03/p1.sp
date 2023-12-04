struct Buffer {
    ptr: I64,
    len: I64,
}

fn get(buffer: Buffer, mut index: I64) -> I64 {
    // bounds check
    if index < 0 || index >= buffer.len {
        exit(1);
    };

    // we are still working with i64s, so multiply by 8
    index = index * 8;

    let mut res = 0i64;
    let ptr = buffer.ptr;
    asm {
        movq {ptr} %RAX
        addq {index} %RAX
        movq [%RAX + 0] {res}
    };
    res
}

fn read_all_to_buffer() -> Buffer {
    let size = 0x10_00_00;
    let start = malloc(size);

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
        if res == b'\0' {
            break;
        };
        ptr = ptr + 8;
        if ptr - start > size {
            exit(1);
        };
    };

    let len = (ptr - start) / 8;

    Buffer {
        ptr: start,
        len,
    }
}

fn main() {
    let buffer = read_all_to_buffer();

    // Find grid dimensions.
    let mut width = 0;
    while get(buffer, width) != b'\n' {
        width = width + 1;
    };
    let mut height = buffer.len / (width + 1);

    let mut total = 0;
    let mut i = 0;
    while i < height {
        let mut number = 0;     // current number we are in (0 if not in number)
        let mut j = 0;          // column index
        let mut start = 0;      // start of current number

        while j < width + 1 {
            let char = get(buffer, i * (width + 1) + j);

            if char >= b'0' && char <= b'9' {
                if number == 0 {
                    start = j;
                };
                number = number * 10 + char - b'0';
            } else {
                if number != 0 {
                    let mut symbol = false; // whether we found a symbol around the number
                    let mut k = max(start - 1, 0);
                    let k_max = min(j + 1, width);

                    while k < k_max {
                        // line above
                        if i != 0 {
                            if is_symbol(get(buffer, (i - 1) * (width + 1) + k)) {
                                symbol = true;
                                break
                            };
                        };
                        // line same level
                        if is_symbol(get(buffer, i * (width + 1) + k)) {
                            symbol = true;
                            break
                        };
                        // line below
                        if i != height - 1 {
                            if is_symbol(get(buffer, (i + 1) * (width + 1) + k)) {
                                symbol = true;
                                break
                            };
                        };
                        k = k + 1;
                    };

                    if symbol {
                        total = total + number;
                    };
                    number = 0; // reset number (no longer in one)
                };
            };
            j = j + 1;
        };
        i = i + 1;
    };
    print(total);
}

fn is_symbol(sym: I64) -> Bool {
    sym != b'.' && (sym < b'0' || sym > b'9')
}
