.global main

main:
  .L0:
    mov x28, 3
    add x28, x28, x28
    mov x0, x28
    ret
