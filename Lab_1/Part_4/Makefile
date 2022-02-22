obj-m += myModSmart.o 
obj-m += myMod.o
myModSmart-objs := MyMem_Smart.o SmartFileOps.o 
myMod-objs := MemManager.o FileOps.o

KDIR := /lib/modules/`uname -r`/build
PWD := $(shell pwd)


all:
	make -C $(KDIR) M=$(PWD) modules

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean
	
test:
	gcc TestProg4.c -o TestProg4

dataCollection:
	gcc DataCollection4.c -o DataCollection4

