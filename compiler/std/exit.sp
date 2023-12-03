fn exit(exit_code: I64) -> Never {
    asm {
        movq {exit_code} %RDI
        movq $60 %RAX // todo: allow parsing other integer formats (e.g. 0x3C)
        syscall 2
    };
    loop { unit }
}
