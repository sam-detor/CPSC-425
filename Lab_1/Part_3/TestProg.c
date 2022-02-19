#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <errno.h>
#include <fcntl.h>

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    char[10] string = "helloWorld"
    char myChar;
    printf("file descriptor %d", fd);
    ioctl(fd,0,10);
    for(i = 0; i< 10; i++)
    {
        write(fd, &(string[i]), 1);
    }
    for(i = 0; i< 10; i++)
    {
        read(fd, &(string[i]), 1);
    }
    int ret = close(fd);
    printf("close return val: %d", ret);
}