#include "MemManager.h"



local_llseek,
local_read,
local_write,
local_ioctl,
local_open,
local_close,

int local_open (struct inode *inode, struct file *flip)
{
    memManagerStructure* dev;
    dev = container_of(inode->i_cdev, memManagerStructure, cdev);
    flip->private_data = dev;
    return 0;
}

int local_close(struct inode* inode, struct file flip)
{
    // also has to deallocate what was allocated in flip->private data
    return 0;
}

ssize_t local_read (struct file flip, const char _ _user *buff, size_t count, loff_t *offp)
{
    if(count != 1)
    {
        return -EINVAL
    }
    struct region* data_region = (flip->private_data)->current_region;
    if(*offp >= data_region->region_size)
    {
        return 0;
    }

    char byteToRead = (data_region->data)[data_region->offset];
    int ret = copy_to_user(buf,&byteToRead,count);
    if(ret == 0)
    {
        return -EFAULT;
    }
    *offp += count;
    data_region->offset += count;

    return count;
}

ssize_t local_write (struct file flip, const char _ _user *buff, size_t count, loff_t *offp)
{
    if(count != 1)
    {
        return -EINVAL
    }

    struct region* data_region = (flip->private_data)->current_region;
    if(*offp >= data_region->region_size)
    {
        return 0;
    }

    int ret = copy_from_user((data_region->data)[data_region->offset], buff, count);
    if(ret == 0)
    {
        return -EFAULT;
    }

    *offp += count;
    data_region->offset += count;
    return count;  
    
}

