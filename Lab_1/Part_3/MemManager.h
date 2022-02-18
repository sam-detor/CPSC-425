#include <linux/init.h>
#include <linux/module.h>
#include <linux/moduleparam.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/kdev_t.h>

struct region
{
    char* data,
    int region_size,
    int region_number,
    int offset
} 

struct myMem_struct
{
    struct region* current_region,
    struct region* data_region,
    int current_region_number,
    int bytes_allocated,
    struct cdev my_cdev

} memManagerStructure;


int local_open (struct inode *inode, struct file *flip);