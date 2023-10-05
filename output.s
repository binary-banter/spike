.globl main
.data
format: .asciz "%d\n"

.text
main:
	pushq %rbp
	movq %rbp, %rsp
	subq %rsp, 16
	jmp core
conclusion:
	addq %rsp, 16
	popq %rbp
	movq %rdi, 0
	call exit
core:
    movq %rdi, [rel format]
    movq %rsi, 18
    xor %eax, %eax
    call printf

	movq [%rbp-8], 1
	movq %rax, [%rbp-8]
	movq [%rbp-16], %rax
	movq [%rbp-24], 1
	movq %rax, [%rbp-16]
	addq [%rbp-24], %rax
	movq %rax, [%rbp-24]
	jmp conclusion
