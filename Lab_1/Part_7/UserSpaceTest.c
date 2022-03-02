#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SYS_CAPITALIZE_NUM (548)

long my_syscall(long sys_num, char* string, int len)
{
    long returnVal;

    __asm__ ("movq %1, %%rax\n\t" //move system call number into rax
                "movq %2, %%rdi\n\t" //move char pointer into rdi
                "movq %3, %%rsi\n\t" //move leng of string into rdi
                "syscall\n\t"      //trigger system call handler
                "movq %%rax, %0"  //get return value
    : "=g" (returnVal) // %0
    : "g"(sys_num), "g"(string), "g"(len) //%1, %2 respectively
    : "rax", "rdi", "rsi"); //this lets the compilier know that these registers get clobbered in this assembly script

    return returnVal;
}

int main(void)
{
    char* testString1 = "hello world!\n";
    char* testString2 = "Hi, ThERe #5!";
    int len1 = strlen(testString1);
    int len2 = strlen(testString2);
    
    //Copying test strings into malloc'ed strings that can be edited
    char* realTestString = malloc(len1);
    realTestString = strcpy(realTestString,testString1);

    char* realTestString2 = malloc(len2);
    realTestString2 = strcpy(realTestString2,testString2);

    printf("Test String 1 Original: %s\n", realTestString);
    printf("Test String 2 Original: %s\n", testString2);

    my_syscall(SYS_CAPITALIZE_NUM, realTestString, len1);
    my_syscall(SYS_CAPITALIZE_NUM, realTestString2, len2);

    printf("Test String 1 Final: %s\n", realTestString);
    printf("Test String 2 Final: %s\n", realTestString2);

    free(realTestString);
    free(realTestString2);
}