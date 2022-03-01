#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>

#define SYS_CAPITALIZE_NUM (548)

int main(void)
{
    char* testString1 = "hello world!";
    char* testString2 = "Hi, ThERe #5!";
    size_t len1 = strlen(testString1);
    size_t len2 = strlen(testString2);

    //Copying test strings into malloc'ed strings that can be edited
    char* realTestString = malloc(len1); 
    realTestString = strcpy(realTestString,testString1); 

    char* realTestString2 = malloc(len2);
    realTestString2 = strcpy(realTestString2,testString2); 

    printf("Test String 1 Original: %s\n", realTestString);
    printf("Test String 2 Original: %s\n", testString2);

    //Calling the syscall
    syscall(SYS_CAPITALIZE_NUM, realTestString, len1);
    syscall(SYS_CAPITALIZE_NUM, realTestString2, len2);

    printf("Test String 1 Final: %s\n", realTestString);
    printf("Test String 2 Final: %s\n", realTestString2);

    free(realTestString);
    free(realTestString2);

    
}