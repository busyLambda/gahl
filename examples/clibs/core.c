#include <unistd.h>
#include <string.h>
#include <stdio.h>
#include <memory.h>

int raw_write(const char *s) {
    size_t len = strlen(s);  // Calculate the length of the string
    int result;

    asm volatile (
        "mov $1, %%rax;"        // syscall number for sys_write (1)
        "mov $1, %%rdi;"        // file descriptor (1 = stdout)
        "mov %1, %%rsi;"        // pointer to the string (second argument)
        "mov %2, %%rdx;"        // length of the string (third argument)
        "syscall;"              // make the system call
        "mov %%eax, %0;"        // store the result in the 'result' variable
        : "=r" (result)         // output
        : "r" (s), "r" (len)    // inputs
        : "%rax", "%rdi", "%rsi", "%rdx"  // clobbered registers
    );

    return result;  // Return the result of the system call (number of bytes written or an error code)
}

int print_number(int *num) {
    printf("Number -> %d\n", *num);
    return 0;
}
