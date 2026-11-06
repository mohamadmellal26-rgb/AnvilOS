global start
extern _start

section .text
bits 32
align 8
header_start:
    dd 0xe85250d6                ; Multiboot2 Magic Number
    dd 0                         ; Architecture 32-bit protected mode
    dd header_end - header_start ; Header length
    dd -(0xe85250d6 + 0 + (header_end - header_start)) ; Checksum
    
    ; --- طلب دقة 720p (1280x720) من خلال Multiboot2 Framebuffer Tag ---
    align 8
    dw 5          ; Type: Framebuffer tag
    dw 0          ; Flags: 0
    dd 20         ; Size: حجم هذا الـ Tag (20 بايت)
    dd 1280       ; Width: العرض 1280 بكسل
    dd 720        ; Height: الارتفاع 720 بكسل
    dd 32         ; Depth: عمق الألوان 32-bit (True Color)

    ; End tag (علامة النهاية الإجبارية)
    align 8
    dw 0
    dw 0
    dd 8
header_end:

start:
    mov esp, stack_top

    ; حفظ مؤشر معلومات Multiboot2 (القادم في ebx) في متغير الذاكرة بشكل آمن
    mov [multiboot_ptr_32], ebx

    ; --- إعداد الصفحات لتغطية 4 جيجابايت بالكامل (باستخدام صفحات 1GB) ---
    mov eax, p3_table
    or eax, 0b11                ; Present + Writable
    mov [p4_table], eax

    ; المدخلة الأولى: 0 إلى 1GB
    mov eax, 0b10000011         ; Present + Writable + Huge Page (1GB)
    mov [p3_table + 0], eax

    ; المدخلة الثانية: 1GB إلى 2GB
    mov eax, 0x40000000
    or eax, 0b10000011
    mov [p3_table + 8], eax

    ; المدخلة الثالثة: 2GB إلى 3GB
    mov eax, 0x80000000
    or eax, 0b10000011
    mov [p3_table + 16], eax

    ; المدخلة الرابعة: 3GB إلى 4GB (تغطي عنوان الـ Framebuffer 0xfd000000)
    mov eax, 0xC0000000
    or eax, 0b10000011
    mov [p3_table + 24], eax

    ; تفعيل PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; تعيين CR3
    mov eax, p4_table
    mov cr3, eax

    ; تفعيل Long Mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; تفعيل Paging
    mov eax, cr0
    or eax, 1 << 31 | 1 << 0
    mov cr0, eax

    ; تحميل GDT
    lgdt [gdt64.pointer]
    jmp gdt64.code_selector:_start_64

section .rodata
gdt64:
    dq 0
.code_selector: equ $ - gdt64
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .text
bits 64
_start_64:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    mov rsp, stack_top
    
    ; استرجاع مؤشر Multiboot2 وتمريره كمعامل أول (rdi) لدالة Rust _start
    mov rax, [multiboot_ptr_32]
    mov rdi, rax
    call _start

.hang:
    cli
    hlt
    jmp .hang

section .bss
align 4096
stack_bottom:
    resb 16384
stack_top:

align 4096
multiboot_ptr_32:
    resq 1

align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096