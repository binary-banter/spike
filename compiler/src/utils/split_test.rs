use std::cell::OnceCell;

/// Splits the inputs, expected outputs and expected return from the test.
/// The values must be preceded by `//*` and `inp:`, `out:` or `ret:`.
#[must_use]
pub fn split_test(test: &str) -> (Vec<i64>, Vec<i64>, i64, Option<&str>) {
    let mut input = OnceCell::new();
    let mut output = OnceCell::new();
    let mut expected_return = OnceCell::new();
    let mut expected_error = OnceCell::new();

    for line in test.lines() {
        let mut parts = line.split_whitespace();

        match (parts.next(), parts.next()) {
            (Some("//*"), Some("inp:")) => input.set(parts.map(str_to_int).collect()).unwrap(),
            (Some("//*"), Some("out:")) => output.set(parts.map(str_to_int).collect()).unwrap(),
            (Some("//*"), Some("ret:")) => expected_return
                .set(parts.next().map(str_to_int).unwrap())
                .unwrap(),
            (Some("//*"), Some("err:")) => expected_error.set(parts.next().unwrap()).unwrap(),
            _ => {}
        }
    }

    (
        input.take().unwrap_or_default(),
        output.take().unwrap_or_default(),
        expected_return.take().unwrap_or(0),
        expected_error.take(),
    )
}

pub fn str_to_int(str: &str) -> i64 {
    match str {
        "true" => 1,
        "false" => 0,
        "unit" => 0,
        _ => str.parse().unwrap(),
    }
}
