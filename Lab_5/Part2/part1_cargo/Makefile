CROSS_COMPILE = arm-none-eabi-
DBG = $(CROSS_COMPILE)gdb
COPY = $(CROSS_COMPILE)objcopy
TARGET = part1_cargo
MYDIR = /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins
SRC = $(MYDIR)/*
NUM = 0
build:
	cargo build --release
	cargo objcopy --bin $(TARGET) --release -- -O binary $(TARGET).bin

burn:
	@st-flash write $(TARGET).bin 0x8000000

debug:
	gdb -q target/thumbv7em-none-eabi/release/$(TARGET) --eval-command="target extended-remote :3333" \

prep:
		for number in 0 1 2 3; for file in $(SRC) do \
			$(COPY) --redefine-sym task=task$$number $$file ; \
		done
prep2: 
	rustc /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/task_format.rs
	./task_format

prep_tasks: 
	#$(COPY) --redefine-sym start=start0 --redefine-sym task_stack_size_getter=task0_stack_size_getter /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_red.a \\
	$(COPY) --redefine-sym start=start1 --redefine-sym task_stack_size_getter=task1_stack_size_getter /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_blue.a
	$(COPY) --redefine-sym start=start2 --redefine-sym task_stack_size_getter=task2_stack_size_getter /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_green.a
	#$(COPY) --redefine-sym start=start3 --redefine-sym task_stack_size_getter=task3_stack_size_getter /Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_orange.a
	
	
	

	


#debug:@$(DBG) -q --eval-command="target extended-remote :4242"


	 