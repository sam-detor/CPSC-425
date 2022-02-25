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
    char* testString2 = "Hi, ThHeRe #5!";
    size_t len1 = strlen(testString1);
    size_t len2 = strlen(testString2);

    printf("Test String 1 Original: %s\n", testString1);
    printf("Test String 2 Original: %s\n", testString2);

    syscall(SYS_CAPITALIZE_NUM, testString1, len1);
    syscall(SYS_CAPITALIZE_NUM, testString2, len2);

    printf("Test String 1 Final: %s\n", testString1);
    printf("Test String 2 Final: %s\n", testString2);

    
}