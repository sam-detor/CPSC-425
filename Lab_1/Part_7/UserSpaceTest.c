#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SYS_CAPITALIZE_NUM (548)

long my_syscall_test(long sys_num, long fd, char* string, size_t len)
{
    long returnVal;
    int src = 1;
    int dst;

    __asm__ ("movq %1, %%rax\n\t"
                "movq %2, %%rdi\n\t"
                "movq %3, %%rsi\n\t"
                "movq %4, %%rdx\n\t"
                "syscall\n\t"
                "movq %%rax, %0"
    : "=g" (returnVal)
    : "g"(sys_num), "g"(fd),"g"(string), "g"(len)
    : "rax", "rdi", "rsi", "rdx");

    return returnVal;
}

long my_syscall(long sys_num, char* string, size_t len)
{
    long returnVal;
    int src = 1;
    int dst;

    __asm__ ("movq %1, %%rax\n\t"
                "movq %2, %%rdi\n\t"
                "movq %3, %%rsi\n\t"
                "syscall\n\t"
                "movq %%rax, %0"
    : "=g" (returnVal)
    : "g"(sys_num), "g"(string), "g"(len)
    : "rax", "rdi", "rsi");

    return returnVal;
}

int main(void)
{
    char* testString1 = "hello world!\n";
    char* testString2 = "Hi, ThHeRe #5!";
    size_t len1 = strlen(testString1);
    size_t len2 = strlen(testString2);
    
    my_syscall_test(1,1,testString1,len1);

    /*
    printf("Test String 1 Original: %s\n", testString1);
    printf("Test String 2 Original: %s\n", testString2);

    my_syscall(SYS_CAPITALIZE_NUM, testString1, len1);
    my_syscall(SYS_CAPITALIZE_NUM, testString2, len2);

    printf("Test String 1 Final: %s\n", testString1);
    printf("Test String 2 Final: %s\n", testString2);
    */
    
}