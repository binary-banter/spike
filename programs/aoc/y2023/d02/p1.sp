fn main() {
    let mut total = 0;
    let mut game = 0;
    let mut next = 0;

    while (next = read_char(); next != b'\0') {
        game = game + 1;
        while (next = read_char(); next != b':') { }

        let mut is_possible = true;

        while next != b'\n' {
            next = read_char(); // space

            let mut number = 0;
            while (next = read_char(); next >= b'0' && next <= b'9') {
                number = number * 10 + next - b'0';
            }

            next = read_char(); // r | g | b

            if next == b'r' {
                if number > 12 {
                    is_possible = false;
                };
                next = read_char(); // e
                next = read_char(); // d
            } else if next == b'g' {
                if number > 13 {
                    is_possible = false;
                };
                next = read_char(); // r
                next = read_char(); // e
                next = read_char(); // e
                next = read_char(); // n
            } else if next == b'b' {
                if number > 14 {
                    is_possible = false;
                };
                next = read_char(); // l
                next = read_char(); // u
                next = read_char(); // e
            } else {
                exit(1)
            }

            next = read_char(); // newline | comma | semicolon
        }

        if is_possible {
            total = total + game;
        }
    }
    print(total);
}
