#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <fcntl.h>

int main()
{
    int fd = open("/dev/mymem", O_RDWR);
    int errorNum = 0;
    char* string = "helloWorld";
    char* string2 = "goodbyeWor";
    char myChar = 'z';
    int size = 10;
    printf("file descriptor %d\n", fd);
    int num1 = ioctl(fd,0,&size);
    printf("num1:%d\n", num1);
    for(int i = 0; i< 10; i++)
    {
        //printf("%c",string[i]);
        write(fd, &(string[i]), 1);
    }
    lseek(fd, 0, 0);
    //printf("%d\n",ioctl(fd,2,num1));
    //perror("The error is:");
    int num2 = ioctl(fd,0,&size);
    printf("num2:%d\n", num2);
    int num3 = ioctl(fd,0,&size);
    printf("num3:%d\n", num3);
    int num4 = ioctl(fd,0,&size);
    int num5 = ioctl(fd,0,&size);
    printf("%d\n",ioctl(fd,1,&num3));
    errorNum = errno;
    
    printf("%d\n",ioctl(fd,2,&num4));
    perror("The error is:");
    for(int i = 0; i< 10; i++)
    {
        //printf("%c",string[i]);
        write(fd, &(string2[i]), 1);
    }
    ioctl(fd,2,&num1);
    lseek(fd, 0, 0);
    for(int i = 0; i < 10; i++)
    {
        read(fd, &myChar, 1);
        printf("%c", myChar);
        myChar = 'z';
    }
    printf("\n");
    ioctl(fd,2,&num5);
    lseek(fd, 0, 0);
    for(int i = 0; i < 10; i++)
    {
        read(fd, &myChar, 1);
        printf("%c", myChar);
        myChar = 'z';
    }
    printf("\n");
    int ret = close(fd);
    
    printf("close return val: %d\n", ret);
}