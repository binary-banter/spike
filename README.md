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
Note that our language returns its expression as the exit code, which is why we use `echo $?`.

Run with input from stdin:
```sh
echo "(+ 1 2)" | cargo run && ./output ; echo $?
```
Run with specified input path, without specifying an output path:
```sh
cargo run -- example.jj && ./example ; echo $?
```
Run with specified input path and specified output path:
```sh
cargo run -- input.jj -o output && ./output ; echo $?
```

PrgParsed
PrgTypeChecked
PrgUniquified
PrgAtomized
PrgExplicated
X86Selected
X86WithInterference
X86Colored
X86Assigned
X86Patched
X86Concluded
X86Emitted (binary)