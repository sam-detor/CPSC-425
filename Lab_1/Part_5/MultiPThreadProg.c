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

#define WORKERS (400)
#define N (100000)

pthread_mutex_t lock;
pthread_t threads[WORKERS];
int fd;

void *forThread(void * ptr)
{
    unsigned long long childCounter;
    for(int j = 0; j < N; j++)
    {
        pthread_mutex_lock(&lock);
        lseek(fd, 0, 0);
        read(fd, &childCounter, 8);
        childCounter++;
        lseek(fd, 0, 0);
        write(fd, &childCounter, 8);
        pthread_mutex_unlock(&lock);
                
    }
    pthread_exit(0);
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

    if (pthread_mutex_init(&lock, NULL) != 0) {
        printf("\n mutex init has failed\n");
        return 1;
    }

    for (int id=0; id < WORKERS; id++) {
       pthread_create(&threads[id],NULL, forThread, NULL);
    }

    for (int id=0; id < WORKERS; id++) {
       pthread_join(threads[id],NULL);
    }
    lseek(fd, 0, 0);
    read(fd, &myCounter, 8);
    close(fd);
    pthread_mutex_destroy(&lock);
    printf("Counter Val: %lld\n", myCounter);
}


