obj-m += myModSmart.o
myModSmart-objs := MyMem_Smart.o SmartFileOps.o

all:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean

