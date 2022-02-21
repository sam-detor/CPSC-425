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

#define WORKERS (2)
#define N (10)

pthread_t threads[WORKERS];
int fd;
int n = N;
int workers = WORKERS;

void *forThread(void * ptr)
{
    unsigned long long childCounter;
    for(int j = 0; j < n; j++)
    {
        //If the call fails, it retries it, as per the ed discussion post
        if(lseek(fd, 0, 0) < 0);
        {
            lseek(fd, 0, 0);
        }
        if(read(fd, &childCounter, 8) < 8)
        {
            read(fd, &childCounter, 8);
        }
        childCounter++;
        if(lseek(fd, 0, 0) < 0);
        {
            lseek(fd, 0, 0);
        }
        if(write(fd, &childCounter, 8) < 8)
        {
            write(fd, &childCounter, 8);
        }
                
    }
    return (void *)0;
}

int main()
{
    int size = 10;

    //Parent Code
    fd = open("/dev/mymem_smart", O_RDWR);
    int num1 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    unsigned long long myCounter = 0;

    if(write(fd,&myCounter, 8) != 8)
    {
        printf("couldn't write");
        return 0;
    }
    lseek(fd,0,0);


    for (int id=0; id < workers; id++) {
       pthread_create(&threads[id],NULL, forThread, NULL);
    }

    for (int id=0; id < workers; id++) {
       pthread_join(threads[id],NULL);
    }
    lseek(fd, 0, 0);
    read(fd, &myCounter, 8);
    close(fd);
    printf("N: %d, W: %d, Counter Val: %lld\n", N, WORKERS, myCounter);
}


