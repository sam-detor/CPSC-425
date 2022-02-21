#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <stdint.h>
#include <sys/wait.h>

#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*)
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)

int main()
{
    pid_t child_pid, wpid;
    int status = 0;
    int size = 0;
    int n = 5;

    //Parent Code
    int fd = open("/dev/mymem_smart", O_RDWR);
    int num1 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    unsigned long long myCounter = 0;
    if(write(fd,&myCounter, 8) != 8)
    {
        printf("couldn't write");
        return 0;
    }
    lseek(fd,0,0);

    for (int id=0; id<n; id++) {
        if ((child_pid = fork()) == 0) {
            unsigned long long childCounter;
            lseek(fd, 0, 0);
            read(fd, &childCounter, 8);
            childCounter++;
            lseek(fd, 0, 0);
            write(fd, &childCounter, 8);
            printf("here!");
            exit(0);
        }
    }

    while ((wpid = wait(&status)) > 0);
    lseek(fd, 0, 0);
    read(fd, &myCounter, 8);
    printf("Counter Val: %lld", myCounter);
}
