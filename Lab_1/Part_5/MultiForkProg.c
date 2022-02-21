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
#include <pthread.h>

#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*)
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)

pthread_mutex_t lock;

int main()
{
    pid_t child_pid, wpid;
    int status = 0;
    int size = 10;
    int workers = 40;
    int n = 100000;

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

    if (pthread_mutex_init(&lock, NULL) != 0) {
        printf("\n mutex init has failed\n");
        return 1;
    }

    for (int id=0; id < workers; id++) {
        if ((child_pid = fork()) == 0) {
            unsigned long long childCounter;
            for(int j = 0; j < n; j++)
            {
                pthread_mutex_lock(&lock);
                lseek(fd, 0, 0);
                read(fd, &childCounter, 8);
                childCounter++;
                lseek(fd, 0, 0);
                write(fd, &childCounter, 8);
                pthread_mutex_unlock(&lock);
                
            }
            exit(0);
        }
    }

    while ((wpid = wait(&status)) > 0);
    lseek(fd, 0, 0);
    read(fd, &myCounter, 8);
    close(fd);
    pthread_mutex_destroy(&lock);
    printf("Counter Val: %lld\n", myCounter);
}
