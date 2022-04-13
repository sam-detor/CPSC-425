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
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;

static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

static STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000100));

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

    //get access to timer
    let tim3 = &stm32f4_peripherals.TIM3;

    //set prescalar value
    //to turn an 8mHz clock into 1ms intervals
    tim3.psc.write(|w| w.psc().bits(15999));

    //set auto refil value
    tim3.arr.write(|w| w.arr().bits(1000));

    //enable interrupts
    tim3.dier.write(|w| w.uie().set_bit());

    //set as a repetitive timer
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    // Set led pins to output
    let gpiod = &stm32f4_peripherals.GPIOD;
    gpiod.moder.write(|w| w.moder14().output());

    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOD
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOD));
    });

    // 7. Enable TIM3 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 2);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

    // Move shared peripherals into mutexes
    cortex_m::interrupt::free(|cs| {

        let led_state: u32 = 0x0;
        let mut stack_prt: u32 = STACK_POINTER.borrow(cs).get();
        let old_stack_prt = cortex_m::register::msp::read();

        //setting up the stack
        unsafe {
            asm!(
               "MSR MSP, {stack_prt}",
               "PUSH {{{led_state}}}",
                "MRS {stack_prt}, MSP",
                "MSR MSP, {old_stack_prt}",
                stack_prt = inout(reg) stack_prt,
                led_state = in(reg) led_state,
                old_stack_prt = in(reg) old_stack_prt,

            );
        }
        STACK_POINTER.borrow(cs).set(stack_prt);
    });

    //enabling the timer
    tim3.cr1.write(|w| w.cen().set_bit());

    //moving the timer into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
    });

    loop {}
}

#[interrupt]

fn TIM3() {
    //triggeres every 1s, blinks leds based on PLAYING and MY_COLOR

    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut gpiod), &mut Some(ref mut tim3)) = (
            MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            let mut stack_prt = STACK_POINTER.borrow(cs).get();
            let mut led_state: u32;
            let mut old_stack_prt = cortex_m::register::msp::read();

            //getting the led_state off the stack
            unsafe {
                asm!(
                    "MSR MSP, {stack_prt}",
                    "POP {{{led_state}}}",
                    "MRS {stack_prt}, MSP",
                    "MSR MSP, {old_stack_prt}",
                    stack_prt = inout(reg) stack_prt,
                    led_state = out(reg) led_state,
                    old_stack_prt = in(reg) old_stack_prt,

                );
            }

            //setting the led based on "led_state", updating led_state
            if led_state == 1 {
                gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                led_state = 0;
            } else {
                gpiod.odr.modify(|_, w| w.odr14().set_bit());
                led_state = 1;
            }

            //putting the correct led_state back on the stack
            old_stack_prt = cortex_m::register::msp::read();
            unsafe {
                asm!(
                    "MSR MSP, {stack_prt}",
                    "PUSH {{{led_state}}}",
                    "MRS {stack_prt}, MSP",
                    "MSR MSP, {old_stack_prt}",
                    led_state = in(reg) led_state,
                    old_stack_prt = in(reg) old_stack_prt,
                    stack_prt = in(reg) stack_prt,

                );
            }
            //updating the stack pointer
            STACK_POINTER.borrow(cs).set(stack_prt);
        }
    });
}
