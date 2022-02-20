#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    char* string = "helloWorld";
    char myChar = 'z';
    printf("file descriptor %d\n", fd);
    ioctl(fd,0,10);
    for(int i = 0; i< 10; i++)
    {
        //printf("%c",string[i]);
        write(fd, &(string[i]), 1);
    }
    lseek(fd, 0, 0);
    for(int i = 0; i < 10; i++)
    {
        read(fd, &myChar, 1);
        printf("%c", myChar);
        myChar = 'z';
    }
    int ret = close(fd);
    printf("\n");
    printf("close return val: %d\n", ret);
}