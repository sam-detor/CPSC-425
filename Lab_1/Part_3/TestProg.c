#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <errno.h>
#include <fcntl.h>

int main()
{
    int fd = syscall(SYS_open, "/dev/mymem", O_RDWR);
    printf("file descriptor %d", fd);
    int ret = syscall(SYS_close, fd);
    printf("close return val: %d", ret);
}