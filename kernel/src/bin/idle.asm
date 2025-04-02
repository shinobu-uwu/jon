; idle.asm - A simple idle task for a kernel
; To compile: nasm -f elf64 idle.asm -o idle.o

section .text
global _start

_start:
    ; Set up a simple idle loop
    
    ; Enable interrupts
    sti
    
idle_loop:
    ; HLT instruction - halts the CPU until an interrupt arrives
    hlt
    
    ; After interrupt occurs, just loop back
    jmp idle_loop
    
    ; Should never reach here
    xor rax, rax
    ret
