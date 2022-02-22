obj-m += myMod.o
myMod-objs := MemManager.o FileOps.o

all:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean

test:
	gcc TestProg.c -o TestProg

dataCollection:
	gcc DataCollection.c -o DataCollection

