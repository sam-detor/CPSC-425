#include <linux/init.h>
#include <linux/module.h>
#include <linux/moduleparam.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/kdev_t.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("1 byte memory manager. See lab report for sources");
                    
MODULE_VERSION("1.0");

char* deviceName = "mymem";
dev_t devNums;
unsigned int count = 1;

struct file_operations memManager_fops = 
{
    .owner = THIS_MODULE,
    .llseek = local_llseek,
    .read = local_read,
    .write = local_write,
    .ioctl = local_ioctl,
    .open = local_open,
    .close = local_close,
}

struct cdev* my_cdev;
static struct class *myClass;


static int __init memManagerInit(void) //the initialization method that runs when the module is loaded into the kernel
{    
    int ret = alloc_chrdev_region(&devNums, 0, count, deviceName);
    if(ret < 0)
    {
        return ret;
    }
    myClass = class_create(THIS_MODULE,"mymem_class");
    if(myClass == NULL)
    {
        unregister_chrdev_region(devNums, count);
        return -1;
    }
    ret = device_create(myClass, NULL, devNums, NULL, "mymem");
    if(ret == NULL)
    {
        class_destroy(myClass)
        unregister_chrdev_region(devNums, count);
        return -1;
    }

    /*my_cdev = cdev_alloc();
    my_cdev->ops = &memManager_fops;
    my_cdev->owner = THIS_MODULE;
    ret = cdev_add
    if(ret < 0)
    {
        return ret;
    }*/


    return 0;
}

static void __exit memManagerExit(void) //the method that runs when the module is removed from the kernel.
{
    device_destroy(myClass,devNums);
    class_destroy(myClass);
    unregister_chrdev_region(devNums,count);
}

module_init(memManagerInit);
module_exit(memManagerExit);