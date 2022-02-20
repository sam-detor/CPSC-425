#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>
#include <time.h>

#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*)
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)
#define KB (1024)
#define STRING_SIZE (512*KB)

float msecWrite = 0;
float msecRead = 0;
float msecWriteAvg = 0;
float msecReadAvg = 0;
clock_t before;

int main()
{
    printf("IN MILI SECONDS\n");
    int fd = open("/dev/mymem_smart", O_RDWR);
    int size = 524288;
    int num1 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    char* myString = malloc(STRING_SIZE);
    for(int j = 0; j < STRING_SIZE ; j++)
    {
        myString[j] = rand() % 255;
    }
    for(int j = 0; j< 15; j++)
    {
        before = clock();
        for(int i = 0; i< size; i += STRING_SIZE)
        {
            write(fd, myString, STRING_SIZE);
        }
        clock_t difference = clock() - before;
        msecWrite = (float)difference /(float)CLOCKS_PER_SEC * 1000;
        msecWriteAvg += msecWrite;
        lseek(fd, 0, 0);
        before = clock();

        for(int i = 0; i < size; i+= STRING_SIZE)
        {
            read(fd, myString, STRING_SIZE);
        }
        difference = clock() - before;
        msecRead = (float)difference /(float)CLOCKS_PER_SEC * 1000;
        msecReadAvg += msecRead;
        printf("MiliSec to read: %f, MiliSec to Write: %f\n", msecRead, msecWrite);
        lseek(fd, 0, 0);
    }
    msecReadAvg/= 15;
    msecWriteAvg/= 15;
    printf("Avg milisec to read: %f, Avg milisec to Write: %f\n", msecReadAvg, msecWriteAvg);

    int ret = close(fd);
}