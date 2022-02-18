#include <stdio.h>
#include <unistad.h>
#include <sys/sycall.h>
#include <errno.h>

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    printf("file descriptor %d", fd);
    int ret = close(fd);
    printf("close return val: %d", ret);
}