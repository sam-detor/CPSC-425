#include <linux/init.h>
#include <linux/module.h>
#include <linux/moduleparam.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("This module prints out hello world on start-up and goodbye world on finishing depending on the state of the"
                    "enable_logging parameter. It also doubles the parameter double_me when it is changed by the user. See lab"
                    "for sources.");
                    
MODULE_VERSION("1.0");

static int enable_logging = 1;
module_param(enable_logging, int, S_IRUGO | S_IWUSR); //creates the enable logging module parameter with user read/write permissions

static int double_me = 0;


static int double_val(const char* val, const struct kernel_param *kp) //the method that runs to write user input to the double_me module param
{                                                                     
    int ret = param_set_int(val, kp);
    if(ret == 0)
    {
        int oldVal = double_me;
        double_me *= 2;
        if(enable_logging)
        {
            printk(KERN_INFO "Initial Value: %d Doubled Value: %d \n", oldVal, double_me);
        }

    }
    return EINVAL;
}

const struct kernel_param_ops double_me_ops = //the methods that run when the double_me param is set or requested
{
    .set = &double_val,
    .get = param_get_int,
};

module_param_cb(double_me, &double_me_ops, &double_me, S_IRUSR | S_IWUSR); //creates the double_me module parameter that notifies the
                                                                           //module when the value has been changed
static int __init helloWord(void) //the initialization method that runs when the module is loaded into the kernel
{
    if(enable_logging)
    {
        printk(KERN_INFO "Hello, World!\n");
    }
    
    return 0;
}

static void __exit goodbyeWorld(void) //the method that runs when the module is removed from the kernel.
{
    if(enable_logging)
    {
        printk(KERN_INFO "Goodbye, World!\n");
    }
    
}

module_init(helloWord);
module_exit(goodbyeWorld);