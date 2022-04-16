#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(type_ascription)]

//#[allow(unused_imports)]
extern crate cortex_m;
extern crate cortex_m_rt as rt;
use panic_halt as _;
extern crate stm32f4;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f411::{self};

use core::arch::asm;
use core::borrow::Borrow;
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;
/* 
static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

static BLUE_STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000300));
static RED_STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000700));
static BLUE_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static RED_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static BLUE_RUNNING: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
*/
static mut MUTEX_GPIOD:stm32f4::stm32f411::GPIOD = None;

static MUTEX_TIM3: stm32f4::stm32f411::TIM3 = None;

static mut BLUE_STACK_POINTER: u32 = 0x20000300;
static mut RED_STACK_POINTER: u32 = 0x20000700;
static BLUE_COUNTER: u32 = 0;
static RED_COUNTER: u32 = 0;
static WHOSE_RUNNING: u32 = 0;

fn FlashBlue() { //0x80007f4 to 0x8000cd6
    //PC: 0x80009b0
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                let counter = BLUE_COUNTER.borrow(cs).get();
                if counter >= 100 {
                    //100 milisec in a sec
                    //led state stuff
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

fn FlashRed() { //0x8000558 to 0x8000e74
    //PC: 0x8000ba6
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                let counter = RED_COUNTER.borrow(cs).get();
                if counter >= 100 {
                    //100 milisec in a sec
                    //led state stuff
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

    // Enabling GPIOC, GPIOD and SYSCFG clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr.write(|w| w.gpioden().set_bit());
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());

    //Enable tim3 and tim4 clocks
    rcc.apb1enr.write(|w| w.tim3en().set_bit());

    //get access to timers
    let tim3 = &stm32f4_peripherals.TIM3;

    //set prescalar values
    //to turn an 8mHz clock into 1ms intervals
    tim3.psc.write(|w| w.psc().bits(15999));

    //set auto refil values
    tim3.arr.write(|w| w.arr().bits(10));

    //enable interrupts
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

    // 7. Enable EXTI7 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

    // Move shared peripherals into mutexes
    cortex_m::interrupt::free(|cs| {
        //set up stack
        let mut blue_stack_prt: u32 = BLUE_STACK_POINTER.borrow(cs).get();
        let mut red_stack_prt: u32 = RED_STACK_POINTER.borrow(cs).get();
        let xpcr = 1 << 24;
        let lr_exception_blue = FlashBlue as u32; //| (1 << 0);
        let lr_exception_red = FlashRed as u32; //| (1 << 0);
        let dummy_val = 0;
        let all_regs_lr: u32 = 0xFFFFFFF9;

        //set up blue stack
        unsafe {
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
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "MRS {blue_stack_prt}, MSP",
               "MSR MSP, {old_stack_prt}",
                blue_stack_prt = inout(reg) blue_stack_prt,
                xpcr = in(reg) xpcr,
                lr_exception_blue = in(reg) lr_exception_blue,
                old_stack_prt = out (reg) _,
                dummy_val = in(reg) dummy_val,
                //all_regs_lr = in(reg) all_regs_lr,

            );
        }

         //set up red stack
         unsafe {
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
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "PUSH {{{dummy_val}}}",
               "MRS {red_stack_prt}, MSP",
               "MSR MSP, {old_stack_prt}",
                red_stack_prt = inout(reg) red_stack_prt,
                xpcr = in(reg) xpcr,
                lr_exception_red = in(reg) lr_exception_red,
                old_stack_prt = out (reg) _,
                dummy_val = in(reg) dummy_val,
                all_regs_lr = in(reg) all_regs_lr,

            );
        }
    
        BLUE_STACK_POINTER.borrow(cs).set(blue_stack_prt);
        RED_STACK_POINTER.borrow(cs).set(red_stack_prt);

    });
    //enabling the timers
    tim3.cr1.write(|w| w.cen().set_bit());

    //moving the timers into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
    });

    FlashBlue();
    //FlashRed();
    loop {}
}
//r0 - r3
#[interrupt]
fn TIM3() {
    //triggeres every 0.5s, blinks leds based on PLAYING and MY_COLOR

    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim3) = 
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut()
        {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            let blue_stack_prt: u32 = BLUE_STACK_POINTER.borrow(cs).get();
            let red_stack_prt: u32 = RED_STACK_POINTER.borrow(cs).get();

            RED_COUNTER.borrow(cs).set(RED_COUNTER.borrow(cs).get() + 1);
            BLUE_COUNTER.borrow(cs).set(BLUE_COUNTER.borrow(cs).get() + 1);

            let blue_running = BLUE_RUNNING.borrow(cs).get();

            if blue_running {
                BLUE_STACK_POINTER.borrow(cs).set(blue_stack_prt - (14 * 4));
                RED_STACK_POINTER.borrow(cs).set(red_stack_prt + (14 * 4));
                BLUE_RUNNING.borrow(cs).set(false);
                
                unsafe{
                contex_switch(blue_stack_prt,red_stack_prt);
                }
                
            }
            else
            {
                BLUE_STACK_POINTER.borrow(cs).set(blue_stack_prt + (14 * 4));
                RED_STACK_POINTER.borrow(cs).set(red_stack_prt - (14 * 4));
                BLUE_RUNNING.borrow(cs).set(true);
                
                unsafe{
                contex_switch(red_stack_prt,blue_stack_prt);
                }

            }


            
        }
    });
}

#[naked]
#[no_mangle]
  pub unsafe extern "C" fn contex_switch(old_ptr: u32, new_ptr: u32) { //pointer in r0, pc in r1
    asm! (
        "MSR MSP, r0",
        "PUSH {{r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}}",
        "PUSH {{r14}}",
        "MSR MSP, r1",
        "POP {{r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12}}",
        "POP {{r14}}",
        //"MOV r0, {lr}",
        //"MOV r14, r0",
        "blx LR",
       // lr = const 0xfffffff1 : u32,
        options(noreturn),
    );
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn trampoline() {
    asm!(
        "bl ",
        "ldr lr, =0xfffffff9",
        "bx lr"
    )
}

fn a_rust_function ()
{

}
//use bx
//disable to prevent nested
//contex switch
///bx lr
