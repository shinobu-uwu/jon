; test.asm
BITS 64
DEFAULT REL     ; Use relative addressing
section .data
    msg db 'Hello, kernel!', 0  ; Null-terminated string
    len equ $ - msg             ; Length of the string

section .text
global _start

_start:
    ; Load syscall number for sys_print into rax
    mov rax, 1                  ; Assuming sys_print has number 1

    ; Load pointer to the string (msg) into rdi
    lea rdi, [msg]              ; Address of msg

    ; Load length of the string into rsi
    mov rsi, len                ; Length of msg

    ; Call the syscall (sys_print)
    syscall
two:
    nop
    jmp two
