# How to Use

```
Usage: rust_compiler_construction.exe [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Specifies the path to an input .jj file. If None, it means stdin is used for input

Options:
  -o, --output <OUTPUT>  Specifies the path to an output file. If None, it uses the input filename. If that's also None, it defaults to "output"
  -h, --help             Print help
  -V, --version          Print version

```

# Examples

These examples demonstrate how to build and run the compiler using Cargo.

Run with input from stdin:

```sh
echo "fn main() { print(42); }" | cargo run && ./output
```

Run with specified input path, without specifying an output path:

```sh
cargo run -- example.sp && ./example
```

Run with specified input path and specified output path:

```sh
cargo run -- input.jj -o output && ./output
```

# Language Features

* Literals
* Var
* BinaryOp
* UnaryOp
* Let
* If
* Functions
  * Return
* Loop
  * Break
  * Continue
* While
* Sequences
* Structs

# AoC Wishlist for Christmas

* Fixing that not very nice bug.
* Pass raw strings for testing (allows testing AoC problems).
* Compile + run

# Fixes

* [ ] Updated README, with 3 new colors!
* [ ] Add documentation where necessary.
* [x] Improve error handling for parsing pass.
* [x] Improve error handling for type checking pass.
  * [ ] Make errors prettier.
* [ ] Improve algorithm for colouring the interference graph.
* [x] Add read and write functionality to the bencher to update locally.
* [x] Lots, and lots, of refactoring!
* [x] Write test input in comments.

# Upcoming Language Features

* [x] Type inference.
* [x] Implement comments in code.
* [ ] Algebraic Data Types.
  * [x] Structs.
  * [ ] Enums.
* [ ] First-class functions.
* [ ] Constants.

# Upcoming Optimizations

* [ ] Dead code.
* [ ] Constant folding.
* [ ] Improve prologue and epilogue.
* [ ] Tail calls.
* [ ] And probably more...

# Lofty Goals

* [ ] Make the compiler suggest hints.
* [ ] Nested definitions.
* [ ] Match statements.
* [ ] LSP.
