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

float msecWrite = 0;
float msecRead = 0;
float msecWriteAvg = 0;
float msecReadAvg = 0;
clock_t before;

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    int size = 524288;
    int num1 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    for(int j = 0; j< 15; j++)
    {
        before = clock();
        for(int i = 0; i< size; i++)
        {
            char toWrite = rand() % 255;
            write(fd, &(toWrite), 1);
        }
        clock_t difference = clock() - before;
        msecWrite = (float)difference /(float)CLOCKS_PER_SEC;
        msecWriteAvg += msecWrite;
        lseek(fd, 0, 0);
        before = clock();
        char myChar;
        for(int i = 0; i < size; i++)
        {
            read(fd, &myChar, 1);
        }
        difference = clock() - before;
        msecRead = (float)difference /(float)CLOCKS_PER_SEC;
        msecReadAvg += msecRead;
        printf("Seconds to read: %f, Seconds to Write: %f\n", msecRead, msecWrite);
    }
    msecReadAvg/= 15;
    msecWriteAvg/= 15;
    printf("Avg seconds to read: %f, Avg seconds to Write: %f\n", msecReadAvg, msecWriteAvg);

    int ret = close(fd);
}