//* err: MismatchedLoop
fn main() {
    loop {
        break 5i64;
        break true;
    };
}
