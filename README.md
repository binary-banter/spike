```shell
cargo run && gcc output.s -no-pie -Wa,-msyntax=intel  -o output && ./output
```

todo:
* write X86 syscall exit to test out elf-header > program-header > program text structure
* emit to X86