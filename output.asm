global	_start
section	.text
conclusion:
	add qword rsp, 16
	pop qword rbp
mov qword rax, 60
mov qword rdi, 0
syscall
main:
	mov qword [rbp-8], 1
	mov qword rax, [rbp-8]
	mov qword [rbp-16], rax
	mov qword [rbp-24], 1
	mov qword rax, [rbp-16]
	add qword [rbp-24], rax
	mov qword rax, [rbp-24]
	jmp conclusion
_start:
	push qword rbp
	mov qword rbp, rsp
	sub qword rsp, 16
	jmp main
