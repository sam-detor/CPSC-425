#include <linux/init.h>
#include <linux/module.h>
#include <linux/moduleparam.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/kdev_t.h>
#include <linux/uaccess.h>
#include <linux/ioctl.h>
#include <linux/cdev.h>
#include <linux/slab.h>

#define MYMEM_IOCTL_ALLOC (0)
#define MYMEM_IOCTL_FREE (1)
#define MYMEM_IOCTL_SETREGION (2)
#define MAX_MEM (1048576)

struct region
{
    char* data;
    unsigned int region_size;
    int region_number;
    int offset;
    struct region* next;
};

struct myMem_struct
{
    struct region* current_region;
    struct region* data_region;
    int current_region_number;
    unsigned int bytes_allocated;
    struct cdev my_cdev;

};


int local_open (struct inode *inode, struct file *flip);
int local_close(struct inode* inode, struct file *filp);
ssize_t local_read (struct file* filp, char __user *buff, size_t count, loff_t *offp);
ssize_t local_write (struct file* filp, const char __user *buff, size_t count, loff_t *offp);
loff_t local_llseek(struct file * filp, loff_t off, int whence);
long int local_ioctl(struct file* filp, unsigned int cmd, unsigned long arg);