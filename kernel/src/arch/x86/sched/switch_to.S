section .text
global switch_context
; switch_context(prev: *mut TaskContext, next: *mut TaskContext)
; rdi = prev context pointer
; rsi = next context pointer
switch_context:
    ; Save current registers to prev context
    mov [rdi + 0],  rsp   ; save stack pointer
    mov [rdi + 8],  rbp   ; save base pointer
    mov [rdi + 16], rbx   ; save callee-saved registers
    mov [rdi + 24], r12
    mov [rdi + 32], r13
    mov [rdi + 40], r14
    mov [rdi + 48], r15
    ; Save return address (the instruction after 'call switch_context')
    mov rax, [rsp]
    mov [rdi + 56], rax

    ; Load next context's registers
    mov rsp, [rsi + 0]    ; restore stack pointer
    mov rbp, [rsi + 8]    ; restore base pointer
    mov rbx, [rsi + 16]   ; restore callee-saved registers
    mov r12, [rsi + 24]
    mov r13, [rsi + 32]
    mov r14, [rsi + 40]
    mov r15, [rsi + 48]
    mov rax, [rsi + 56]   ; get new instruction pointer

    ; Jump to new task
    ; We push the target address and return to it
    push rax
    ret
