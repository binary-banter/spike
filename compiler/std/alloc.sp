fn malloc(size: I64) -> I64 {
    let PROT_READ = 0x1i64;
    let PROT_WRITE = 0x2i64;
    let MAP_PRIVATE = 0x02i64;
    let MAP_ANONYMOUS = 0x20i64;

    // Julia says hi! Also sorry for saying '+' is equivalent to '|' here.
    let prot = PROT_READ + PROT_READ;
    let flags = MAP_PRIVATE + MAP_ANONYMOUS;
    let fd: I64 = -1i64;

    let mut ptr = 0i64;
    asm {
        movq $9 %RAX        // mmap
        movq $0 %RDI        // address hint (none) - this means linux gives us an address in ptr
        movq {size} %RSI    // size to be allocated in bytes
        movq {prot} %RDX    // allow read and write
        movq {flags} %R10   // private and anonymous mapping
        movq {fd} %R8       // file descriptor of -1 is needed on some systems when using anonymous map (usually ignored)
        movq $0 %R9         // offset
        syscall 7
        movq %RAX {ptr}
    };
    if ptr < 0 {
        print(ptr);
        exit(-1);
    };
    ptr
}
