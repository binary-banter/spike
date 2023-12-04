fn main() {
    let mut total = 0;
    let mut first = 0;
    let mut last = 0;

    let mut next = 0;
    while (next = read_char(); next != b'\0') {
        if next == b'\n' {
            total = total + first * 10 + last;
            first = 0;
        }
        next = next - b'0';
        if next >= 0 && next < 10 {
            if first == 0 {
                first = next;
            }
            last = next;
        }
    }
    print(total);
}
