#![cfg(unix)]

use std::path::Path;
use test_each_file::test_each_path;

fn aoc([solution, output]: [&Path; 2]) {
    todo!()
}

test_each_path! { for ["sp, out"] in "./programs/aoc/" as aoc => aoc }
