CROSS_COMPILE = arm-none-eabi-
DBG = $(CROSS_COMPILE)gdb
TARGET = part1_cargo
build:
	cargo build --release
	cargo objcopy --bin $(TARGET) --release -- -O binary $(TARGET).bin

burn:
	@st-flash write $(TARGET).bin 0x8000000

debug:
	@$(DBG) -q --eval-command="target extended-remote :4242" \
	 /Users/samdetor/CPSC-425/CPSC-425/Lab_3/Part1/part2_cargo/target/thumbv7em-none-eabi/debug/part2_cargo