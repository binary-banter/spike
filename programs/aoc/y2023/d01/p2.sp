fn main() {
    let mut total = 0;
    let mut first = 0;
    let mut last = 0;

    let mut x0 = 0;
    let mut x1 = 0;
    let mut x2 = 0;
    let mut x3 = 0;
    let mut x4 = 0;

    let mut next = 0;
    while (next = read_char(); next != b'\0') {
        x0 = x1;
        x1 = x2;
        x2 = x3;
        x3 = x4;
        x4 = next;

        if next == b'\n' {
            total = total + first * 10 + last;
            first = 0;
            continue
        }

        let digit = (if x2 == b'o' && x3 == b'n' && x4 == b'e' { 1 }
        else if x2 == b't' && x3 == b'w' && x4 == b'o' { 2 }
        else if x0 == b't' && x1 == b'h' && x2 == b'r' && x3 == b'e' && x4 == b'e' { 3 }
        else if x1 == b'f' && x2 == b'o' && x3 == b'u' && x4 == b'r' { 4 }
        else if x1 == b'f' && x2 == b'i' && x3 == b'v' && x4 == b'e' { 5 }
        else if x2 == b's' && x3 == b'i' && x4 == b'x' { 6 }
        else if x0 == b's' && x1 == b'e' && x2 == b'v' && x3 == b'e' && x4 == b'n' { 7 }
        else if x0 == b'e' && x1 == b'i' && x2 == b'g' && x3 == b'h' && x4 == b't' { 8 }
        else if x1 == b'n' && x2 == b'i' && x3 == b'n' && x4 == b'e' { 9 }
        else if next >= b'0' && next <= b'9' { next - b'0' }
        else { continue });

        if first == 0 {
            first = digit;
        }
        last = digit;
    }
    print(total);
}
