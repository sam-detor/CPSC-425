#include "MemManager.h"


/*
local_llseek,
local_read,
local_write,
local_ioctl,
local_open,
local_close,
*/

static int nextID = 0;

int local_open (struct inode *inode, struct file *filp)
{
    struct myMem_struct* dev = container_of(inode->i_cdev, struct myMem_struct, my_cdev);
    filp->private_data = dev;
    return 0;
}

int local_close(struct inode* inode, struct file filp)
{
    // also has to deallocate what was allocated in filp->private data
    return 0;
}

ssize_t local_read (struct file* filp, char __user *buff, size_t count, loff_t *offp)
{
    struct myMem_struct* dev = (filp->private_data);
    struct region* data_region = dev->current_region;
    char byteToRead;
    int ret;

    if(count != 1)
    {
        return -EINVAL;
    }


    if(data_region == NULL)
    {
        return 0;
    }

    if(*offp >= data_region->region_size)
    {
        return 0;
    }

    
    byteToRead = (data_region->data)[data_region->offset];
    ret = copy_to_user(buff,&byteToRead,count);
    if(ret == 0)
    {
        return -EFAULT;
    }
    *offp += count;
    data_region->offset += count;

    return count;
}

ssize_t local_write (struct file* filp, const char __user *buff, size_t count, loff_t *offp)
{

    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    int ret;

    if(count != 1)
    {
        return -EINVAL;
    }

    if(data_region == NULL)
    {
        return 0;
    }

    if(*offp >= data_region->region_size)
    {
        return 0;
    }

    ret = copy_from_user(&((data_region->data)[data_region->offset]), buff, count);
    if(ret == 0)
    {
        return -EFAULT;
    }

    *offp += count;
    data_region->offset += count;
    return count;  
    
}

loff_t local_llseek(struct file * filp, loff_t off, int whence)
{
    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    loff_t newPos;

    switch(whence)
    {
        case 0: 
        newPos = off;
        break;

        case 1:
        newPos = data_region->offset + off;
        break;

        case 2:
        newPos = data_region->region_size + off;
        break;

        default:
        return -EINVAL;
    }

    if(newPos > 0 && newPos <= data_region->region_size)
    {
        filp->f_pos = newPos;
        data_region->offset = newPos;
        return newPos;

    }
    else
    {
        return -EINVAL;
    }
}

int local_ioctl(struct file* filp, unsigned int cmd, unsigned long arg)
{
    struct myMem_struct* dev = filp->private_data;
    struct region* head = dev->data_region;
    struct region* new_region;
    char* allocated_data;
    struct region* temp;
    struct region* temp_prev;
    int regionNum;
    switch(cmd)
    {
        case MYMEM_IOCTL_ALLOC:
        if(dev->bytes_allocated + (unsigned int)arg > MAX_MEM)
        {
            return -ENOMEM;
        }

        new_region = kmalloc(sizeof(struct region), GFP_KERNEL);
        if(new_region == NULL)
        {
            return -ENOMEM;
        }
        allocated_data = kmalloc(arg, GFP_KERNEL);
        if(allocated_data == NULL)
        {
            return -ENOMEM;
        }

        temp = head;
        
        if(dev->data_region == NULL)
        {
            dev->data_region = new_region;
            dev->current_region = new_region;
            dev->current_region_number = nextID;
            new_region->region_number = nextID;
            nextID++;
            new_region->next = NULL;
            new_region->offset = 0;
            new_region->data = allocated_data;
            new_region->region_size = (unsigned int)arg;
            dev->bytes_allocated = (unsigned int)arg;
            return 0;
        }
        
        while(temp->next != NULL)
        {
            temp = temp->next;
        }
        temp->next = new_region;
        new_region->region_number = nextID;
        nextID++;
        new_region->next = NULL;
        new_region->offset = 0;
        new_region->data = allocated_data;
        new_region->region_size = (unsigned int)arg;
        dev->bytes_allocated += (unsigned int)arg;
        return new_region->region_number;
        break;

        case MYMEM_IOCTL_FREE:
        
        temp = head;
        regionNum = (int)arg;
        while(temp != NULL && temp->region_number != regionNum)
        {
            temp_prev = temp;
            temp = temp->next;
        }
        if(temp == NULL)
        {
            return -EINVAL;
        }
        temp_prev->next = temp->next;
        dev->bytes_allocated -= temp->region_size;
        kfree(temp->data); 
        return 0;
        break;
        
        case MYMEM_IOCTL_SETREGION:
        return -ENOTTY;
        break;

        default:
        return -ENOTTY;

    }
} 

