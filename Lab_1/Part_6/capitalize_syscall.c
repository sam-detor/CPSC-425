#include <linux/kernel.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/syscalls.h>

//SYSCALL 548!!

SYSCALL_DEFINE2(capitalize, char __user*, buff, int, length)
{
    int ret;
    char* editableString = kmalloc(length, GFP_KERNEL);
    if(editableString == NULL)
    {
        return -EINVAL;
    }
    int i;
    ret = copy_from_user(editableString, buff, length);
    if(ret != 0)
    {
        return -1;
    }

    printk(KERN_INFO "Input String: %s\n", editableString);

    for(i = 0; i < length; i++)
    {
        if(editableString[i] >= 'a' && editableString[i] <= 'z')
        {
            editableString[i] = editableString[i] - 32;
        }
    }

    ret = copy_to_user(buff, editableString, length);
    if(ret != 0)
    {
        return -1;
    }
    kfree(editableString);
    return 0;
}
