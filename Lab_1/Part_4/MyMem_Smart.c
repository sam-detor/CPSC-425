#include "MyMem_Smart.h"

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("Smarter memory manager. Depends on the base module (myMod). See lab report for sources");
                    
MODULE_VERSION("1.0");

//declaring global vars
char* deviceName = "mymem_smart";
dev_t devNums;
unsigned int count = 1;

struct file_operations memManagerSmart_fops = 
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
    int ret = alloc_chrdev_region(&devNums, 0, count, deviceName); //allocating major and minor numbers for the device
    if(ret < 0)
    {
        printk(KERN_INFO "unable to allocate region");
        return ret;
    }

    //initializing the cdev struct
    cdev_init(&(mymem.my_cdev), &memManagerSmart_fops);
    mymem.my_cdev.ops = &memManagerSmart_fops;
    mymem.my_cdev.owner = THIS_MODULE;

    //adding the cdev struct to the allocated character device region previously, now it needs to be able to take requests
    ret = cdev_add(&(mymem.my_cdev), devNums, count);
    if(ret < 0)
    {
        printk(KERN_INFO "unable to create the add cdev");
        unregister_chrdev_region(devNums, count);
        return ret;
    }

    //creating the device class
    myClass = class_create(THIS_MODULE,"mymem_smart_class");
    if(myClass == NULL)
    {
        printk(KERN_INFO "unable to create the class");
        cdev_del(&(mymem.my_cdev));
        unregister_chrdev_region(devNums, count);
        return -1;
    }

    //creating the device file
    myDev = device_create(myClass, NULL, devNums, NULL, "mymem_smart");
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