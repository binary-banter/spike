.globl main
.text
core:
	movq [%rbp-8], 1
	movq %rax, [%rbp-8]
	movq [%rbp-16], %rax
	movq [%rbp-24], 1
	movq %rax, [%rbp-16]
	addq [%rbp-24], %rax
	movq %rax, [%rbp-24]
	jmp conclusion
conclusion:
	addq %rsp, 16
	popq %rbp
	movq %rax, 60
	movq %rdi, 0
	syscall
main:
	pushq %rbp
	movq %rbp, %rsp
	subq %rsp, 16
	jmp core
