CROSS_COMPILE = arm-none-eabi-
DBG = $(CROSS_COMPILE)gdb
COPY = $(CROSS_COMPILE)objcopy
TARGET = part2_cargo
MYDIR = /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins
NUM = 0
build: prep_tasks
	cargo build --release
	cargo objcopy --bin $(TARGET) --release -- -O binary $(TARGET).bin

burn:
	@st-flash write $(TARGET).bin 0x8000000

debug:
	gdb -q target/thumbv7em-none-eabi/release/$(TARGET) --eval-command="target extended-remote :3333" \

prep_tasks: 
	$(COPY) --redefine-sym start=start0 --redefine-sym task_stack_size=task0_stack_size $(MYDIR)/libflash_red.a 
	$(COPY) --redefine-sym start=start1 --redefine-sym task_stack_size=task1_stack_size $(MYDIR)/libflash_blue.a
	$(COPY) --redefine-sym start=start2 --redefine-sym task_stack_size=task2_stack_size $(MYDIR)/libflash_green.a
	$(COPY) --redefine-sym start=start3 --redefine-sym task_stack_size=task3_stack_size $(MYDIR)/libflash_orange.a


	 