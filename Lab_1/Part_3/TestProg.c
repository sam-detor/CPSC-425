#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>

#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*)
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)

int main()
{
    //opening file and declaring useful vars
    int fd = open("/dev/mymem", O_RDWR);
    int errorNum = 0;
    char* string = "helloWorld";
    char* string2 = "goodbyeWor";
    char myChar = 'z';
    int size = 10;
    
    printf("file descriptor %d\n", fd);

    int num1 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size); //Allocating a region 
    printf("num1:%d\n", num1);

    for(int i = 0; i< 10; i++) //and writing to it
    {
        write(fd, &(string[i]), 1);
    }
    lseek(fd, 0, 0);

    int num2 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size); //Allocating a bunch of regions
    printf("num2:%d\n", num2);

    int num3 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    printf("num3:%d\n", num3);

    int num4 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);
    int num5 = ioctl(fd,MYMEM_IOCTL_ALLOC,&size);

    printf("ioctl free region return val: %d\n",ioctl(fd,MYMEM_IOCTL_FREE,&num3)); //testing ioctl free region function
  
    
    printf("ioctl set region return val: %d\n",ioctl(fd,MYMEM_IOCTL_SETREGION,&num4)); //testing ioctl set region function
 
    for(int i = 0; i< 10; i++) //writing to the newly set region
    {
        write(fd, &(string2[i]), 1);
    }

    ioctl(fd,MYMEM_IOCTL_SETREGION,&num1);
    lseek(fd, 0, 0);

    for(int i = 0; i < 10; i++) //reading from the original region
    {
        read(fd, &myChar, 1);
        printf("%c", myChar);
        myChar = 'z'; //to catch failed reads
    }
    printf("\n");

    ioctl(fd,MYMEM_IOCTL_SETREGION,&num4);
    lseek(fd, 0, 0);

    for(int i = 0; i < 10; i++) //reading from the other region written to
    {
        read(fd, &myChar, 1);
        printf("%c", myChar);
        myChar = 'z'; //to catch failed reads
    }
    printf("\n");

    //Testing Mod params

    int syfs_fd = open("/sys/module/myMod/parameters/regions", O_RDONLY);
    printf("file descriptor:%d, path: %s \n", syfs_fd, "/sys/module/myMod/parameters/regions");

    if(syfs_fd > 0)
    {
        char sysfs;
        int read_val  = read(syfs_fd, &sysfs, 1);
        while(read_val == 1) //reading all the chars in the regions param file
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
        int read_val = read(param_file, &paramChar, 1);
        while(read_val == 1) //reading all the chars in the allocated param file
        {
            printf("%c", paramChar);
            read_val  =  read(param_file, &paramChar, 1);
        }
        printf("\n");
    }

    int ret = close(fd);
    
    printf("close return val: %d\n", ret);
}