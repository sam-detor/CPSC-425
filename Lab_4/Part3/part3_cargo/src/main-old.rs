#![no_std]
#![no_main]

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

static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

static BLUE_STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000100));
static RED_STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000500));
static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0)); 

fn FlashBlue() { //PC: 0x80009b0
    static mut led_state: u32 = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = 
                MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut()
            {
                let mut stack_prt = BLUE_STACK_POINTER.borrow(cs).get();
                let counter = COUNTER.borrow(cs).get

                unsafe {
                    asm!(
                        "MRS {old_stack_prt}, MSP",
                        "MSR MSP, {stack_prt}",
                        "POP {{{led_state}}}",
                        "MRS {stack_prt}, MSP",
                        "MSR MSP, {old_stack_prt}",
                        stack_prt = inout(reg) stack_prt,
                        led_state = out(reg) led_state,
                        old_stack_prt = out(reg) _,

                    );
                }

                BLUE_STACK_POINTER.borrow(cs).set(stack_prt);

                //led state stuff
                if led_state == 1 {
                    gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                    led_state = 0;
                } else {
                    gpiod.odr.modify(|_, w| w.odr15().set_bit());
                    led_state = 1;
                }

                unsafe {
                    asm!(
                        "MRS {old_stack_prt}, MSP",
                        "MSR MSP, {stack_prt}",
                        "PUSH {{{led_state}}}",
                        "MRS {stack_prt}, MSP",
                        "MSR MSP, {old_stack_prt}",
                        led_state = in(reg) led_state,
                        old_stack_prt = out(reg) _,
                        stack_prt = in(reg) stack_prt,

                    );
                }
                BLUE_STACK_POINTER.borrow(cs).set(stack_prt);
            }
        });
        
        cortex_m::asm::delay(8000000);
    }
}

fn FlashRed() { //PC: 0x8000ba6
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = 
                MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut()
            {
                let mut stack_prt = RED_STACK_POINTER.borrow(cs).get();
                let mut led_state: u32 = 0;

                unsafe {
                    asm!(
                        "MRS {old_stack_prt}, MSP",
                        "MSR MSP, {stack_prt}",
                        "POP {{{led_state}}}",
                        "MRS {stack_prt}, MSP",
                        "MSR MSP, {old_stack_prt}",
                        stack_prt = inout(reg) stack_prt,
                        led_state = out(reg) led_state,
                        old_stack_prt = out(reg) _,

                    );
                }

                RED_STACK_POINTER.borrow(cs).set(stack_prt);

                //led state stuff
                if led_state == 1 {
                    gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                    led_state = 0;
                } else {
                    gpiod.odr.modify(|_, w| w.odr14().set_bit());
                    led_state = 1;
                }

                unsafe {
                    asm!(
                        "MRS {old_stack_prt}, MSP",
                        "MSR MSP, {stack_prt}",
                        "PUSH {{{led_state}}}",
                        "MRS {stack_prt}, MSP",
                        "MSR MSP, {old_stack_prt}",
                        led_state = in(reg) led_state,
                        old_stack_prt = out(reg) _,
                        stack_prt = in(reg) stack_prt,

                    );
                }
                RED_STACK_POINTER.borrow(cs).set(stack_prt);
            }
        });
        
        cortex_m::asm::delay(8000000);
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
    gpiod
        .moder
        .write(|w| w.moder15().output().moder14().output().moder13().output().moder12().output());

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
        let mut led_state: u32 = 0x0;
        let mut blue_stack_prt: u32 = BLUE_STACK_POINTER.borrow(cs).get();
        let mut red_stack_prt: u32 = RED_STACK_POINTER.borrow(cs).get();
        let blue_status = 1;
        let red_status = 0;
        let blue_pc = 0x80009b0;
        let red_pc = 0x8000ba6;
 
        unsafe {
            asm!(
               "MRS {old_stack_prt}, MSP",
               "MSR MSP, {blue_stack_prt}",
               "PUSH {{{blue_pc}}}",
               "PUSH {{{blue_status}}}",
               "PUSH {{{led_state}}}",
                "MRS {blue_stack_prt}, MSP",
                "MSR MSP, {red_stack_prt}",
                "PUSH {{{red_pc}}}",
                "PUSH {{{red_status}}}",
                "PUSH {{{led_state}}}",
                "MRS {red_stack_prt}, MSP",
                "MSR MSP, {old_stack_prt}",
                blue_stack_prt = inout(reg) blue_stack_prt,
                red_stack_prt = inout(reg) red_stack_prt,
                led_state = in(reg) led_state,
                old_stack_prt = out(reg) _,
                blue_status = in(reg) blue_status,
                red_status = in(reg) red_status,
                blue_pc = in(reg) blue_pc,
                red_pc= in(reg) red_pc,
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
    FlashRed();
    loop {
       
    }
}
//r0 - r3
#[interrupt]

fn TIM3() {
    //triggeres every 0.5s, blinks leds based on PLAYING and MY_COLOR

    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut gpiod), &mut Some(ref mut tim3)) = (
            MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            let mut blue_stack_prt: u32 = BLUE_STACK_POINTER.borrow(cs).get();
            let mut red_stack_prt: u32 = RED_STACK_POINTER.borrow(cs).get();
            
            let mut blue_led_state = 1;
            let mut blue_status = 0;
            let mut blue_pc = 0;

            let mut red_led_state = 1;
            let mut red_status = 0;
            let mut red_pc = 0;

            unsafe {
                asm!(
                    //unwinding both stacks
                   "MRS {old_stack_prt}, MSP",
                   "MSR MSP, {blue_stack_prt}",
                   "POP {{{blue_led_state}}}",
                   "POP {{{blue_status}}}",
                   "POP {{{blue_pc}}}",
                    "MRS {blue_stack_prt}, MSP",
                    "MSR MSP, {red_stack_prt}",
                    "POP {{{red_led_state}}}",
                    "POP {{{red_status}}}",
                    "POP {{{red_pc}}}",
                    "MRS {red_stack_prt}, MSP",
                    "MSR MSP, {old_stack_prt}",
                    blue_stack_prt = inout(reg) blue_stack_prt,
                    red_stack_prt = inout(reg) red_stack_prt,
                    blue_led_state = out(reg) _,
                    red_led_state = out(reg) _,
                    old_stack_prt = out(reg) _,
                    blue_status = out(reg) _,
                    red_status = out(reg) _,
                    blue_pc = out(reg) _,
                    red_pc= out(reg) _,
                );
            }
        
             let mut exit_pc: u32;
            if red_status == 0 {

                //blue_pc = pc;
                blue_status = 0;
                red_status = 1;
                exit_pc = red_pc;
                //gpiod.odr.modify(|_,w| w.odr13().set_bit());
            }
            else if blue_status == 0 {
                //red_pc = pc;
                red_status = 0;
                blue_status = 1;
                exit_pc = blue_pc;
                //gpiod.odr.modify(|_,w| w.odr12().set_bit());
            }
            else {
                exit_pc = blue_pc;
                //gpiod.odr.modify(|_,w| w.odr14().set_bit());
            }
            let mut pc = 0; 
           
            //finding PC
            unsafe {
                asm!(
                    //unwinding both stacks
                   "POP {{{tmp1}}}",
                   "POP {{{tmp2}}}",
                   "POP {{{tmp3}}}",
                   "POP {{{tmp4}}}",
                   "POP {{{tmp5}}}",
                   "POP {{{tmp6}}}",
                   "POP {{{pc}}}",
                   "PUSH {{{exit_pc}}}",
                   "PUSH {{{tmp6}}}",
                   "PUSH {{{tmp5}}}",
                   "PUSH {{{tmp4}}}",
                   "PUSH {{{tmp3}}}",
                   "PUSH {{{tmp2}}}",
                   "PUSH {{{tmp1}}}",
                    tmp1 = out(reg) _,
                    tmp2 = out(reg) _,
                    tmp3 = out(reg) _,
                    tmp4 = out(reg) _,
                    tmp5 = out(reg) _,
                    tmp6 = out(reg) _,
                    pc = out(reg) pc,
                    exit_pc = in(reg) exit_pc,
                );
            }

            if red_status == 0 {

               blue_pc = pc;
                //gpiod.odr.modify(|_,w| w.odr13().set_bit());
            }
            else if blue_status == 0 {
                red_pc = pc;
                //gpiod.odr.modify(|_,w| w.odr12().set_bit());
            }

             
             //re-forming both stacks and exiting
            unsafe {
                asm!(
                   "MRS {old_stack_prt}, MSP",
                   "MSR MSP, {blue_stack_prt}",
                   "PUSH {{{blue_pc}}}",
                   "PUSH {{{blue_status}}}",
                   "PUSH {{{blue_led_state}}}",
                    "MRS {blue_stack_prt}, MSP",
                    "MSR MSP, {red_stack_prt}",
                    "PUSH {{{red_pc}}}",
                    "PUSH {{{red_status}}}",
                    "PUSH {{{red_led_state}}}",
                    "MRS {red_stack_prt}, MSP",
                    "MSR MSP, {old_stack_prt}",
                    blue_stack_prt = inout(reg) blue_stack_prt,
                    red_stack_prt = inout(reg) red_stack_prt,
                    blue_led_state = in(reg) blue_led_state,
                    red_led_state = in(reg) red_led_state,
                    old_stack_prt = out(reg) _,
                    blue_status = in(reg) blue_status,
                    red_status = in(reg) red_status,
                    blue_pc = in(reg) blue_pc,
                    red_pc= in(reg) red_pc,
                );
            }  
        
        }});
}



