#include "MyMem_Smart.h"


ssize_t smart_read (struct file* filp, char __user *buff, size_t count, loff_t *offp) //reads multiple byte from the current offset in the currrent region to the user
{

    struct myMem_struct* dev = (filp->private_data);
    struct region* data_region = dev->current_region;
    char* bytesToRead;
    int ret;

    if(count + data_region->offset > data_region->region_size) //checks if read is within the bound of current region
    {
        return -EINVAL;
    }

    if(data_region == NULL) //returns 0 if no current allocations
    {
        return 0;
    }

    if(*offp >= data_region->region_size) //returns 0 if offset is at the end of the allocated region
    {
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

ssize_t smart_write (struct file* filp, const char __user *buff, size_t count, loff_t *offp) //writes multiple byte to the current offset in the currrent region from the user
{
    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    int ret;

    if(count + data_region->offset > data_region->region_size) //checks if write is within the bound of current region
    {
        
        return -EINVAL;
    }

    if(data_region == NULL) //returns 0 if no current allocations
    {
        
        return 0;
    }

    if(*offp >= data_region->region_size) //returns 0 if offset is at the end of the allocated region
    {
        
        return 0;
    }

    ret = copy_from_user(((data_region->data) + data_region->offset), buff, count); 
    if(ret != 0)
    {

        return -EFAULT;
    }

    *offp += count;
    data_region->offset += count;
    
    return count;  
    
}