struct Buffer {
    ptr: I64,
    len: I64,
}

fn get(buffer: Buffer, mut index: I64) -> I64 {
    // bounds check
    if index < 0 || index >= buffer.len {
        exit(1);
    }

    // we are still working with i64s, so multiply by 8
    index = index * 8;

    let mut res = 0i64;
    let ptr = buffer.ptr;
    asm {
        movq {ptr} %RAX
        addq {index} %RAX
        movq [%RAX + 0] {res}
    }
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
        }
        if res == b'\0' {
            break;
        }
        ptr = ptr + 8;
        if ptr - start > size {
            exit(1);
        }
    }

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
    }
    let mut height = buffer.len / (width + 1);

    let mut total = 0;
    let mut i = 0;
    while i < height {
        let mut j = 0;

        while j < width + 1 {
            let char = get(buffer, i * (width + 1) + j);

            if char == b'*' {
                let mut neighbors = 0i64;
                let mut product = 1;

                let left = find_number(buffer, width, i, j-1);
                if left > 0 {
                    neighbors = neighbors + 1;
                    product = product * left;
                }

                let right = find_number(buffer, width, i, j+1);
                if right > 0 {
                    neighbors = neighbors + 1;
                    product = product * right;
                }

                let top_left = find_number(buffer, width, i-1, j-1);
                if top_left > 0 {
                    neighbors = neighbors + 1;
                    product = product * top_left;
                }

                let top_mid = find_number(buffer, width, i-1, j);
                if top_left == 0 && top_mid > 0 {
                    neighbors = neighbors + 1;
                    product = product * top_mid;
                }

                let top_right = find_number(buffer, width, i-1, j+1);
                if top_mid == 0 && top_right > 0 {
                    neighbors = neighbors + 1;
                    product = product * top_right;
                }

                let bottom_left = find_number(buffer, width, i+1, j-1);
                if bottom_left > 0 {
                    neighbors = neighbors + 1;
                    product = product * bottom_left;
                }

                let bottom_mid = find_number(buffer, width, i+1, j);
                if bottom_left == 0 && bottom_mid > 0 {
                    neighbors = neighbors + 1;
                    product = product * bottom_mid;
                }

                let bottom_right = find_number(buffer, width, i+1, j+1);
                if bottom_mid == 0 && bottom_right > 0 {
                    neighbors = neighbors + 1;
                    product = product * bottom_right;
                }

                if neighbors == 2 {
                    total = total + product;
                }
            }
            j = j + 1;
        }
        i = i + 1;
    }
    print(total);
}

fn find_number(buffer: Buffer, width: I64, i: I64, j: I64) -> I64 {
    // verify there is a number here
    let mut char = get(buffer, i * (width + 1) + j);
    if char < b'0' || char > b'9' {
        return 0
    }

    // find start by scanning left
    let mut x = j;
    loop {
        let char = get(buffer, i * (width + 1) + x);
        if char < b'0' || char > b'9' {
            x = x + 1;
            break
        }
        if x == 0 {
            break
        }
        x = x - 1;
    }

    // find number
    let mut number = 0;
    while (char = get(buffer, i * (width + 1) + x); char >= b'0' && char <= b'9') {
        number = number * 10 + char - b'0';
        x = x + 1;
        if x >= width {
            break
        }
    }

    number
}
