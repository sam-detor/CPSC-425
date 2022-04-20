#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(type_ascription)]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
use panic_halt as _;
extern crate stm32f4;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f411::{self};

use core::arch::asm;
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;

static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

static BLUE_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static RED_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

static mut BLUE_STACK_POINTER: u32 = 0x20000300;
static mut RED_STACK_POINTER: u32 = 0x20000700;

fn flash_blue() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                let counter = BLUE_COUNTER.borrow(cs).get(); //counter gets updated in TIM3
                if counter >= 100 {
                    //100 10 milisec chunks in a sec
                    //toggle led
                    if led_state == 1 {
                        gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                        led_state = 0;
                    } else {
                        gpiod.odr.modify(|_, w| w.odr15().set_bit());
                        led_state = 1;
                    }
                    BLUE_COUNTER.borrow(cs).set(0);
                }
            }
        });
    }
}

fn flash_red() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                let counter = RED_COUNTER.borrow(cs).get(); //counter gets updated in TIM3
                if counter >= 100 {
                    //100 10 milisec chunks in a sec
                    //toggle led
                    if led_state == 1 {
                        gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                        led_state = 0;
                    } else {
                        gpiod.odr.modify(|_, w| w.odr14().set_bit());
                        led_state = 1;
                    }
                    RED_COUNTER.borrow(cs).set(0);
                }
            }
        });
    }
}

#[entry]
fn main() -> ! {
    // Getting access to the peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // Enabling GPIOD and SYSCFG clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr.write(|w| w.gpioden().set_bit());
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());

    //Enable tim3 clock
    rcc.apb1enr.write(|w| w.tim3en().set_bit());

    //get access to timer 3
    let tim3 = &stm32f4_peripherals.TIM3;

    //set prescalar value
    //to turn an 8mHz clock into 1ms intervals
    tim3.psc.write(|w| w.psc().bits(15999));

    //set auto refil value
    tim3.arr.write(|w| w.arr().bits(10));

    //enable interrupt
    tim3.dier.write(|w| w.uie().set_bit());

    //set as a repetitive timer
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    // Set led pins to output
    let gpiod = &stm32f4_peripherals.GPIOD;
    gpiod.moder.write(|w| {
        w.moder15()
            .output()
            .moder14()
            .output()
            .moder13()
            .output()
            .moder12()
            .output()
    });

    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOD
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOD));
    });

    // 7. Enable TIM3 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

    // setting up the red and blue stacks
    unsafe {
        let mut blue_stack_prt: u32 = BLUE_STACK_POINTER;
        let mut red_stack_prt: u32 = RED_STACK_POINTER;
        let xpcr = 1 << 24; //24th bit is 1
        let lr_exception_blue = flash_blue as u32;
        let lr_exception_red = flash_red as u32;
        let dummy_val = 0;
        let all_regs_lr: u32 = trampoline as u32; //initial LR is the trampoline function

        //set up blue stack
        asm!(
            "MRS {old_stack_prt}, MSP",
            "MSR MSP, {blue_stack_prt}",
            "PUSH {{{xpcr}}}",
            "PUSH {{{lr_exception_blue}}}",
            "PUSH {{{lr_exception_blue}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{all_regs_lr}}}",
            "PUSH {{r4-r11}}",
            "MRS {blue_stack_prt}, MSP",
            "MSR MSP, {old_stack_prt}",
            blue_stack_prt = inout(reg) blue_stack_prt,
            xpcr = in(reg) xpcr,
            lr_exception_blue = in(reg) lr_exception_blue,
            old_stack_prt = out (reg) _,
            dummy_val = in(reg) dummy_val,
            all_regs_lr = in(reg) all_regs_lr,

        );

        //set up red stack
        asm!(
            "MRS {old_stack_prt}, MSP",
            "MSR MSP, {red_stack_prt}",
            "PUSH {{{xpcr}}}",
            "PUSH {{{lr_exception_red}}}",
            "PUSH {{{lr_exception_red}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{all_regs_lr}}}",
            "PUSH {{r4-r11}}",
            "MRS {red_stack_prt}, MSP",
            "MSR MSP, {old_stack_prt}",
            red_stack_prt = inout(reg) red_stack_prt,
            xpcr = in(reg) xpcr,
            lr_exception_red = in(reg) lr_exception_red,
            old_stack_prt = out (reg) _,
            dummy_val = in(reg) dummy_val,
            all_regs_lr = in(reg) all_regs_lr,

        );

        // update stack pointer global vars
        BLUE_STACK_POINTER = blue_stack_prt;
        RED_STACK_POINTER = red_stack_prt;
    }

    //enabling the timer
    tim3.cr1.write(|w| w.cen().set_bit());

    //moving the timer into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
    });

    flash_blue();
    loop {}
}

#[interrupt]
fn TIM3() {
    //triggeres every 10ms, switches the executing task
    static mut WHOSE_RUNNING: u32 = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim3) = MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut() {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            //update the timer counters for red and blue
            RED_COUNTER
                .borrow(cs)
                .set(RED_COUNTER.borrow(cs).get() + 1);
            BLUE_COUNTER
                .borrow(cs)
                .set(BLUE_COUNTER.borrow(cs).get() + 1);
        }
    });

    //runs a context switch function based on what task is currently running
    unsafe {
        if *WHOSE_RUNNING == 0 { //initial switch from main to task 1
            *WHOSE_RUNNING = 1;
            context_switch_orig(BLUE_STACK_POINTER, &BLUE_STACK_POINTER);
        } else if *WHOSE_RUNNING == 1 { //task 1 --> task 2
            *WHOSE_RUNNING = 2;
            context_switch(&BLUE_STACK_POINTER, RED_STACK_POINTER, &RED_STACK_POINTER);
        } else { //task 2 --> task 1
            *WHOSE_RUNNING = 1;
            context_switch(&RED_STACK_POINTER, BLUE_STACK_POINTER, &BLUE_STACK_POINTER);
        }
    }
}

#[naked]
#[no_mangle]
/* 
    This function performs the context switch between two tasks, it:
    - saves the current LR and registers r4-11 to the old stack 
    - saves the current value of the sp to the right entry in the task pointers array
    - switches to the new stack
    - restores LR and registers r4-11 from the values in the new stack
    - saves the new sp to the right entry of the task pointers array
    - returns with the restored LR value
*/
pub unsafe extern "C" fn context_switch(old_addy: &u32, new_ptr: u32, new_addy: &u32) {
    asm!(
        "PUSH {{LR}}",
        "PUSH {{r4-r11}}",
        "STR SP, [r0]",
        "MSR MSP, r1",
        "POP {{r4-r11}}",
        "POP {{LR}}",
        "STR SP, [r2]",
        "bx LR",
        options(noreturn),
    );
}

#[naked]
#[no_mangle]
/* 
    This function performs the initial context switch the original main stack
    and the first task, it:
    - switches to the new stack
    - restores LR and registers r4-11 from the values in the new stack
    - saves the new sp to the right entry of the task pointers array
    - returns with the restored LR value
*/
pub unsafe extern "C" fn context_switch_orig(ptr: u32, prt_addy: &u32) { //ptr in r0, addy in r1
    asm!(
        "MSR MSP, r0",
        "POP {{r4-r11}}",
        "POP {{LR}}",
        "STR SP, [r1]",
        "blx LR",
        options(noreturn),
    );
}

#[naked]
#[no_mangle]
/* This is the function that is loaded into the LR value of the initial "saved
registers" portion of each stack. It performs an exception return */
pub unsafe extern "C" fn trampoline() {
    asm!("ldr lr, =0xfffffff9", "bx lr", options(noreturn),);
}
