#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>

#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*)
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    int errorNum = 0;
    char* string = "helloWorld";
    char* string2 = "goodbyeWor";
    char myChar = 'z';
    int size = 10;
    printf("file descriptor %d\n", fd);
    int num1 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    printf("num1:%d\n", num1);
    write(fd, string, 10);
    lseek(fd, 0, 0);
    //printf("%d\n",ioctl(fd,2,num1));
    //perror("The error is:");
    int num2 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    printf("num2:%d\n", num2);
    int num3 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    printf("num3:%d\n", num3);
    int num4 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    int num5 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    printf("%d\n",ioctl(fd,MYMEM_IOCTL_FREE,&num3));
    errorNum = errno;
    
    printf("%d\n",ioctl(fd,MYMEM_IOCTL_SETREGION,&num4));
    perror("The error is:");
    write(fd, string2, 1);
    
    ioctl(fd,MYMEM_IOCTL_SETREGION,&num1);
    lseek(fd, 0, 0);
    char* myString1 = malloc(10);
    read(fd, myString1, 10);
    printf("%s", myString1);
    myChar = 'z';
    printf("\n");
    ioctl(fd,MYMEM_IOCTL_SETREGION,&num4);
    lseek(fd, 0, 0);
    read(fd, myString1, 1);
    printf("%s", myString1);
    printf("\n");
    int syfs_fd = open("/sys/kernel/regions/dataRegions", O_RDONLY);
    printf("file des:%d\n", syfs_fd);
    if(syfs_fd > 0)
    {
        char sysfs;
        int read_val  = read(syfs_fd, &sysfs, 1);
        while(read_val == 1)
        {
            printf("%c", sysfs);
            read_val  = read(syfs_fd, &sysfs, 1);
        }
        printf("\n");
    }
    int param_file = open("/sys/module/myMod/parameters/param_bytes_allocated", O_RDONLY);
    if(syfs_fd > 0)
    {
        char paramChar;
        int read_val  = read(param_file, &paramChar, 1);
        while(read_val == 1)
        {
            printf("%c", paramChar);
            read_val  =  read(param_file, &paramChar, 1);
        }
        printf("\n");
    }
    free(myString1);
    int ret = close(fd);
    
    printf("close return val: %d\n", ret);
}