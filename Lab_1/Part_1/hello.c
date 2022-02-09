#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("This module prints out hello world on start-up and goodbye world on finishing. See lab report for sources");
MODULE_VERSION("1.0");


static int __init helloWord(void) //This function runs when the module is loaded into the kernel
{
    printk(KERN_INFO "Hello, World!\n");
    return 0;
}

static void __exit goodbyeWorld(void) //This function runs when the module is removed from the kernel
{
    printk(KERN_INFO "Goodbye, World!\n");
}

module_init(helloWord);
module_exit(goodbyeWorld);