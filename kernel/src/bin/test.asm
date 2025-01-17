; test.asm
BITS 64
DEFAULT REL     ; Use relative addressing
section .text
global _start
_start:
    mov rcx, 100000000
first:
    nop
    dec rcx
    jnz first
second:
    jmp $       ; Jump to current position (infinite loop)
