pub fn expect<T>(b: bool, err: T) -> Result<(), T> {
    b.then_some(()).ok_or(err)
}
