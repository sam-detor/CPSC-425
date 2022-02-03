#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Sam Detor");
MODULE_DESCRIPTION("Printing out hello world on start-up and goodbye world on finishing. Source: Robert Oliver\'s blog post on building a basic kernel module: https://blog.sourcerer.io/writing-a-simple-linux-kernel-module-d9dc3762c234");
MODULE_VERSION("1.0");


static int __init helloWord(void)
{
    printk(KERN_INFO "Hello, World!\n");
    return 0;
}

static void __exit goodbyeWorld(void)
{
    printk(KERN_INFO "Goodbye, World!\n");
}

module_init(helloWord);
module_exit(goodbyeWorld);