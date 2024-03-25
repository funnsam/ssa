.extern putchar
.global main
main:
  .L0:
    sub sp, sp, #192
    str x29, [sp, #0]
    str x30, [sp, #16]
    str x19, [sp, #32]
    str x20, [sp, #48]
    str x21, [sp, #64]
    str x22, [sp, #80]
    str x23, [sp, #96]
    str x24, [sp, #112]
    str x25, [sp, #128]
    str x26, [sp, #144]
    str x27, [sp, #160]
    str x28, [sp, #176]
    mov x29, sp
    mov x28, 97
    mov x27, 10
    mov x0, x28
    bl putchar
    mov x28, x0
    mov x0, x27
    bl putchar
    mov x27, x0
    b .L1
  .L1:
    mov x27, 3
    add x27, x27, x27
    mov x0, x27
    b .epilogue
  .L2:
    ldr x29, [sp, #0]
    ldr x30, [sp, #16]
    ldr x19, [sp, #32]
    ldr x20, [sp, #48]
    ldr x21, [sp, #64]
    ldr x22, [sp, #80]
    ldr x23, [sp, #96]
    ldr x24, [sp, #112]
    ldr x25, [sp, #128]
    ldr x26, [sp, #144]
    ldr x27, [sp, #160]
    ldr x28, [sp, #176]
    add sp, sp, #192
    autibsp
    ret
