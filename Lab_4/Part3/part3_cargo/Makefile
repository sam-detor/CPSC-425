CROSS_COMPILE = arm-none-eabi-
DBG = $(CROSS_COMPILE)gdb
TARGET = part3_cargo
build:
	cargo build --release
	cargo objcopy --bin $(TARGET) --release -- -O binary $(TARGET).bin

burn:
	@st-flash write $(TARGET).bin 0x8000000

debug:
	gdb -q target/thumbv7em-none-eabi/release/part3_cargo --eval-command="target extended-remote :3333" \
	


#debug:@$(DBG) -q --eval-command="target extended-remote :4242"


	 