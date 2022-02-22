#include "MemManager.h"
#define MYMEM_IOCTL_ALLOC _IOW(236,0,int*) //ioctl function definitions
#define MYMEM_IOCTL_FREE _IOW(236,1,int*)
#define MYMEM_IOCTL_SETREGION _IOW(236,2,int*)

//global var declarations
int nextID = 0;
int allocated = 0;
struct region* dataRegions = NULL;

int local_open (struct inode *inode, struct file *filp)
{
    struct myMem_struct* dev = container_of(inode->i_cdev, struct myMem_struct, my_cdev);
    filp->private_data = dev; //saving the device file struct into the file pointer private memory
    dataRegions = NULL;
    allocated = 0;
    return 0;
}

int local_close(struct inode* inode, struct file* filp) // deallocates all allocated regions and allocated data within those regions
{
    
    struct myMem_struct* dev = (struct myMem_struct*) filp->private_data;
    struct region* head = dev->data_region;
    struct region* temp;

    //reseting the values in the myMem_struct, the global id counter, and the allocated linked list global var
    dev->data_region = NULL;
    dev->current_region = NULL;
    dev->current_region_number = 0;
    nextID = 0;
    dev->bytes_allocated = 0;
    allocated = 0;
    dataRegions = NULL;

    //freeing allocated data
    while(head != NULL)
    {
        temp = head->next;
        kfree(head->data);
        kfree(head);
        head = temp;
    }
    
    printk(KERN_INFO "close!");
    return 0;
}

ssize_t local_read (struct file* filp, char __user *buff, size_t count, loff_t *offp) //reads one byte from the current offset in the currrent region to the user
{

    struct myMem_struct* dev = (filp->private_data);
    struct region* data_region = dev->current_region;
    char byteToRead;
    int ret;

    if(count != 1) //make sure the user is only asking for 1 byte
    {
        return -EINVAL;
    }


    if(data_region == NULL) //return 0 if no allocated regions
    {
        return 0;
    }

    if(*offp >= data_region->region_size) //return 0 if offset is at end of allocated region
    {
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

ssize_t local_write (struct file* filp, const char __user *buff, size_t count, loff_t *offp) //writes one byte to the current offset in the currrent region from the user
{

    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    int ret;
  
    if(count != 1) //makes sure they are only writing 1 byte
    {
        return -EINVAL;
    }

    if(data_region == NULL) //returns 0 if no allocated region
    {
        return 0;
    }

    if(*offp >= data_region->region_size) //returns 0 if offset is at the end of the allocated region
    {
        return 0;
    }

    ret = copy_from_user(&((data_region->data)[data_region->offset]), buff, count);
    if(ret != 0)
    {
        return -EFAULT;
    }
    *offp += count;
    data_region->offset += count;
    return count;  
    
}

loff_t local_llseek(struct file * filp, loff_t off, int whence) //sets file offset based on value of whence and off
{
    struct region* data_region = ((struct myMem_struct*)(filp->private_data))->current_region;
    loff_t newPos;

    switch(whence)
    {
        case 0: 
        newPos = off; //SEEK_SET
       
        break;

        case 1: //SEEK_CUR
        newPos = data_region->offset + off;
        break;

        case 2: //SEEK END
        newPos = data_region->region_size + off;
        break;

        default: //error
        return -EINVAL;
    }

    if(newPos >= 0 && newPos <= data_region->region_size) //checks if offset is within the allocated region
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
    ret = copy_from_user(&my_arg, (int*)arg, sizeof(my_arg)); //gets the argument passed by the user
    if(ret <0)
    {
        return ret;
    }

    switch(cmd)
    {
        case MYMEM_IOCTL_ALLOC:
        if(dev->bytes_allocated + my_arg > MAX_MEM) //if new region would exceed max memory limit, return error
        {
            return -ENOMEM;
        }

        new_region = kmalloc(sizeof(struct region), GFP_KERNEL); //allocate region struct
        if(new_region == NULL)
        {
            return -ENOMEM;
        }
        allocated_data = kmalloc(my_arg, GFP_KERNEL); //allocate region data
        if(allocated_data == NULL)
        {
            return -ENOMEM;
        }

        temp = head;
        
        if(dev->data_region == NULL) //if no other allocated regions, make this region the current region
        {
            dev->data_region = new_region; //new region to head of data regions list and current region
            dev->current_region = new_region;
            dev->current_region_number = nextID; //sets region ID based on global counter nextID
            new_region->region_number = nextID;
            nextID++;

            //fills in relevant information into new region (region struct)
            new_region->next = NULL;
            new_region->offset = 0;
            new_region->data = allocated_data;
            new_region->region_size = (unsigned int)my_arg;
            dev->bytes_allocated = (unsigned int)my_arg;
            allocated = my_arg;
            dataRegions = new_region; 
            return 0;
        }
        
        while(temp->next != NULL) //find last entry in allocated linked list
        {
            temp = temp->next;
        }
        temp->next = new_region; //adds new region to linked list

        //fills in relevant information into new region (region struct)
        new_region->region_number = nextID;
        nextID++;
        new_region->next = NULL;
        new_region->offset = 0;
        new_region->data = allocated_data;
        new_region->region_size = (unsigned int)my_arg;
        dev->bytes_allocated += (unsigned int)my_arg;
        allocated += my_arg;
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

        if(temp == NULL) //if no region has this id, return null
        {
            return -EINVAL;
        }

        if(temp_prev == NULL) //if region to be freed was the head, make the next region in the list the head of the linked list
        {
            dev->data_region = temp->next;
            dataRegions = temp->next;
        }

        else //remove the soon to be freed region from the linked list
        {
            temp_prev->next = temp->next;
        }
        //update params
        dev->bytes_allocated -= temp->region_size;
        allocated -= temp->region_size;

        //free region
        kfree(temp->data); 
        kfree(temp);

        return 0;
        break;
        
        case MYMEM_IOCTL_SETREGION:
        regionNum = my_arg;

        if(regionNum == dev->current_region_number) //if the region is already the current region, return 0
        {
            return 0;
        }

        temp = head;
        while(temp != NULL && temp->region_number != regionNum) //find the region with the given ID in the linked list
        {
            temp = temp->next;
        } 

        if(temp == NULL) //if the region doesn't exist, return an error
        {

            return -EINVAL;
        }

        //set current region to found region
        dev->current_region = temp;
        dev->current_region_number = regionNum;
    
        return 0;
        break;

        default:
        return -ENOTTY;

    }
} 

int sysfs_show(char* buf, const struct kernel_param *kp) //get method for the regions param
{
    struct region* temp = dataRegions;
    char* myStr = kmalloc(50, GFP_KERNEL);
    int size = 0;
    int add = 0;
    while(temp != NULL)
    {
        size = sprintf(myStr,"id: %d, size: %d\n", temp->region_number, temp->region_size); //puts relevant region into string form
        sprintf(buf + add, myStr); //adds it to the provided buffer
        add += size;
        temp = temp->next;
    }
    kfree(myStr);
    return add;
}


EXPORT_SYMBOL(local_ioctl);
EXPORT_SYMBOL(local_open);
EXPORT_SYMBOL(local_close);
EXPORT_SYMBOL(nextID);
EXPORT_SYMBOL(allocated);
EXPORT_SYMBOL(dataRegions);
EXPORT_SYMBOL(local_llseek);
EXPORT_SYMBOL(sysfs_show);