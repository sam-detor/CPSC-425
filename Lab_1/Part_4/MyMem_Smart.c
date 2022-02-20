#include "MyMem_Smart.h"

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("1 byte memory manager. See lab report for sources");
                    
MODULE_VERSION("1.0");

int param_bytes_allocated;
module_param(param_bytes_allocated, int, S_IRUGO);


char* deviceName = "mymem_smart";
dev_t devNums;
unsigned int count = 1;

struct file_operations memManager_fops = 
{
    .owner = THIS_MODULE,
    .llseek = local_llseek,
    .read = smart_read,
    .write = smart_write,
    .unlocked_ioctl = local_ioctl,
    .open = local_open,
    .release = local_close,
};

struct myMem_struct mymem;

static struct class *myClass;
static struct device *myDev;


static int __init smartMemManagerInit(void) //the initialization method that runs when the module is loaded into the kernel
{    
    int ret = alloc_chrdev_region(&devNums, 0, count, deviceName);
    if(ret < 0)
    {
        printk(KERN_INFO "unable to allocate region");
        return ret;
    }

    cdev_init(&(mymem.my_cdev), &memManager_fops);
    mymem.my_cdev.ops = &memManager_fops;
    mymem.my_cdev.owner = THIS_MODULE;
    ret = cdev_add(&(mymem.my_cdev), devNums, count);
    if(ret < 0)
    {
        printk(KERN_INFO "unable to create the add cdev");
        unregister_chrdev_region(devNums, count);
        return ret;
    }

    myClass = class_create(THIS_MODULE,"mymem_class");
    if(myClass == NULL)
    {
        printk(KERN_INFO "unable to create the class");
        cdev_del(&(mymem.my_cdev));
        unregister_chrdev_region(devNums, count);
        return -1;
    }
    myDev = device_create(myClass, NULL, devNums, NULL, "mymem");
    if(myDev == NULL)
    {
        printk(KERN_INFO "unable to create device");
        class_destroy(myClass);
        cdev_del(&(mymem.my_cdev));
        unregister_chrdev_region(devNums, count);
        return -1;
    }

    return 0;
}

static void __exit smartMemManagerExit(void) //the method that runs when the module is removed from the kernel.
{
    cdev_del(&(mymem.my_cdev));
    device_destroy(myClass,devNums);
    class_destroy(myClass);
    unregister_chrdev_region(devNums,count);
}

module_init(smartMemManagerInit);
module_exit(smartMemManagerExit);