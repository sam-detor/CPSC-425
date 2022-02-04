#include <linux/init.h>
#include <linux/module.h>
#inclide <linux/moduleparam.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("Printing out hello world on start-up and goodbye world on finishing. Source: Robert Oliver\'s blog post on building a basic kernel module: https://blog.sourcerer.io/writing-a-simple-linux-kernel-module-d9dc3762c234");
MODULE_VERSION("1.0");

static int enableLogging = 1;
module_param(enableLogging, int, S_IRUSR | S_IWUSR);

static int doubleMe = 0;
module_param_cb(doubleMe, int, S_IRUSR | S_IWUSR);

static int double_val(const char* val, const struct kernel_param *kp)
{
    int ret = param_set_int(val, kp);
    if(ret == 0)
    {
        int newValue = doubleMe * 2;
        int ret2 = param_set_int(newValue, kp);
        if(ret2 == 0)
        {
            return 0;
        }
        return -1;
    }
    return EINVAL;
}

const struct kernel_param_ops doubleMe_ops = 
{
    .set = &double_val;
    .get = param_get_int;
}
static int __init helloWord(void)
{
    struct k_object* loggingDir = k_object_create_and_add("enable_logging", )
    if(enableLogging)
    {
        printk(KERN_INFO "Hello, World!\n");
    }
    
    return 0;
}

static void __exit goodbyeWorld(void)
{
    if(enableLogging)
    {
        printk(KERN_INFO "Goodbye, World!\n");
    }
    
}

module_init(helloWord);
module_exit(goodbyeWorld);