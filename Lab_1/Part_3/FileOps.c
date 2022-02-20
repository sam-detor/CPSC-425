#include "MemManager.h"
#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*)
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)

/*
local_llseek,
local_read,
local_write,
local_ioctl,
local_open,
local_close,
*/




int nextID = 0;
int param_bytes_allocated = 0;
struct region* dataRegions = NULL;

EXPORT_SYMBOL(local_llseek);
EXPORT_SYMBOL(local_ioctl);
EXPORT_SYMBOL(local_open);
EXPORT_SYMBOL(local_close);
EXPORT_SYMBOL(nextID);
EXPORT_SYMBOL(param_bytes_allocated);
EXPORT_SYMBOL(dataRegions);


int local_open (struct inode *inode, struct file *filp)
{
    struct myMem_struct* dev = container_of(inode->i_cdev, struct myMem_struct, my_cdev);
    filp->private_data = dev;
    dataRegions = NULL;
    param_bytes_allocated = 0;
    return 0;
}

int local_close(struct inode* inode, struct file* filp)
{
    // also has to deallocate what was allocated in filp->private data
    struct myMem_struct* dev = (struct myMem_struct*) filp->private_data;
    struct region* head = dev->data_region;
    struct region* temp;
    dev->data_region = NULL;
    dev->current_region = NULL;
    dev->current_region_number = 0;
    nextID = 0;
    dev->bytes_allocated = 0;
    param_bytes_allocated = 0;
    while(head != NULL)
    {
        temp = head->next;
        kfree(head->data);
        kfree(head);
        head = temp;
    }
    dataRegions = NULL;
    printk(KERN_INFO "close!");
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
        //printk(KERN_INFO "too long");
        return 0;
    }

    
    byteToRead = (data_region->data)[data_region->offset];
    ret = copy_to_user(buff,&byteToRead,count);
    if(ret != 0)
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
    //printk(KERN_INFO "here");
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
    if(ret != 0)
    {
        //printk(KERN_INFO "here2");
        return -EFAULT;
    }
    //printk(KERN_INFO "offset pre: %d, offp* pre: %lld", data_region->offset, *offp);
    *offp += count;
    data_region->offset += count;
    //printk(KERN_INFO "offset post: %d, offp* post: %lld", data_region->offset, *offp);
    return count;  
    
}

loff_t local_llseek(struct file * filp, loff_t off, int whence)
{
    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    loff_t newPos;
    //printk(KERN_INFO "Called");
    switch(whence)
    {
        case 0: 
        newPos = off;
        //printk(KERN_INFO "Called2");
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

    if(newPos >= 0 && newPos <= data_region->region_size)
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

long int local_ioctl(struct file* filp, unsigned int cmd, unsigned long arg)
{
    struct myMem_struct* dev = filp->private_data;
    struct region* head = dev->data_region;
    struct region* new_region;
    char* allocated_data;
    struct region* temp;
    struct region* temp_prev;
    int my_arg;
    unsigned int regionNum;
    int ret;
    ret = copy_from_user(&my_arg, (int*)arg, sizeof(my_arg));
    if(ret <0)
    {
        return ret;
    }
    printk(KERN_INFO "cmd: %d", cmd);
    switch(cmd)
    {
        case MYMEM_IOCTL_ALLOC:
        if(dev->bytes_allocated + my_arg > MAX_MEM)
        {
            return -ENOMEM;
        }

        new_region = kmalloc(sizeof(struct region), GFP_KERNEL);
        if(new_region == NULL)
        {
            return -ENOMEM;
        }
        allocated_data = kmalloc(my_arg, GFP_KERNEL);
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
            new_region->region_size = (unsigned int)my_arg;
            dev->bytes_allocated = (unsigned int)my_arg;
            param_bytes_allocated = my_arg;
            dataRegions = new_region; 
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
        new_region->region_size = (unsigned int)my_arg;
        dev->bytes_allocated += (unsigned int)my_arg;
        param_bytes_allocated += my_arg;
        return new_region->region_number;
        break;

        case MYMEM_IOCTL_FREE:
        
        temp = head;
        regionNum = (unsigned int)my_arg;
        temp_prev = NULL;
        while(temp != NULL && temp->region_number != regionNum)
        {
            temp_prev = temp;
            temp = temp->next;
        }
        if(temp == NULL)
        {
            return -EINVAL;
        }
        if(temp_prev == NULL)
        {
            dev->data_region = temp->next;
            dataRegions = temp->next;
        }
        else
        {
            temp_prev->next = temp->next;
        }
        dev->bytes_allocated -= temp->region_size;
        param_bytes_allocated -= temp->region_size;
        kfree(temp->data); 
        kfree(temp);
        return 0;
        break;
        
        case MYMEM_IOCTL_SETREGION:
        regionNum = my_arg;
        printk(KERN_INFO "called set region w %d", regionNum);
        if(regionNum == dev->current_region_number)
        {
            return 0;
        }

        temp = head;
        while(temp != NULL && temp->region_number != regionNum)
        {
            temp = temp->next;
        } 
        if(temp == NULL)
        {
            printk(KERN_INFO "didn't find it");
            return -EINVAL;
        }
        dev->current_region = temp;
        dev->current_region_number = regionNum;
        printk(KERN_INFO "set it");
        return 0;
        break;

        default:
        printk(KERN_INFO "default");
        return -ENOTTY;

    }
} 

ssize_t sysfs_show(struct kobject *kobj, struct kobj_attribute * attr, char* buf)
{
    struct region* temp = dataRegions;
    char* myStr = kmalloc(50, GFP_KERNEL);
    int size = 0;
    int add = 0;
    while(temp != NULL)
    {
        size = sprintf(myStr,"id: %d, size: %d\n", temp->region_number, temp->region_size);
        sprintf(buf + add, myStr);
        add += size;
        temp = temp->next;
    }
    return add;
}

ssize_t sysfs_store(struct kobject *kobj, struct kobj_attribute * attr, const char* buf, size_t count)
{
    return 0;
}

