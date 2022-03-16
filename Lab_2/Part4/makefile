TARGET = StateMachine
SRCS = StateMachine.c

LINKER_SCRIPT = ../flash/STM32F411VETx_FLASH.ld

# Generate debug info
DEBUG = 1

# Choose processor
CDEFS  = -DSTM32F411xE 
# Enable FPU
#CDEFS += -D__VFP_FP__

# link math library
#LIBS = -lm

include ../armf4.mk