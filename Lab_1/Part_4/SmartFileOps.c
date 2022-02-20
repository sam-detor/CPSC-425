#include "MyMem_Smart.h"


ssize_t smart_read (struct file* filp, char __user *buff, size_t count, loff_t *offp)
{

    struct myMem_struct* dev = (filp->private_data);
    struct region* data_region = dev->current_region;
    char* bytesToRead;
    int ret;

    if(count + data_region->offset > data_region->region_size)
    {
        return -EINVAL;
    }

    if(data_region == NULL)
    {
        return 0;
    }

    if(*offp >= data_region->region_size)
    {
        //printk(KERN_INFO "too long");
        return 0;
    }

    
    bytesToRead = (data_region->data) + data_region->offset;
    ret = copy_to_user(buff,bytesToRead,count);
    if(ret != 0)
    {
        return -EFAULT;
    }
    *offp += count;
    data_region->offset += count;

    return count;
}

ssize_t smart_write (struct file* filp, const char __user *buff, size_t count, loff_t *offp)
{
    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    int ret;
    printk(KERN_INFO "here4");
    if(count + data_region->offset > data_region->region_size)
    {
        printk(KERN_INFO "offset: %d, size: %d", data_region->offset, data_region->region_size);
        return -EINVAL;
    }

    if(data_region == NULL)
    {
        printk(KERN_INFO "here1");
        return 0;
    }

    if(*offp >= data_region->region_size)
    {
        printk(KERN_INFO "here2");
        return 0;
    }

    ret = copy_from_user(((data_region->data) + data_region->offset), buff, count);
    if(ret != 0)
    {
        //printk(KERN_INFO "here2");
        printk(KERN_INFO "here3");
        return -EFAULT;
    }
    //printk(KERN_INFO "offset pre: %d, offp* pre: %lld", data_region->offset, *offp);
    *offp += count;
    data_region->offset += count;
    //printk(KERN_INFO "offset post: %d, offp* post: %lld", data_region->offset, *offp);
    return count;  
    
}