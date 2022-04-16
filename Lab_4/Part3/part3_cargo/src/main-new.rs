#![feature(prelude_import)]
#![no_std]
#![no_main]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
use panic_halt as _;
extern crate stm32f4;
use core::arch::asm;
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f411;
use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;
static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));
static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));
static BLUE_STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000100));
static RED_STACK_POINTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0x20000500));

fn FlashBlue() {
    loop {
        free(|cs| {
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                let mut stack_prt = BLUE_STACK_POINTER.borrow(cs).get();
                let mut led_state: u32 = 0;
                unsafe {
                    asm ! ("MRS {2}, MSP\nMSR MSP, {0}\nPOP {{{1}}}\nMRS {0}, MSP\nMSR MSP, {2}" , inout (reg) stack_prt , out (reg) led_state , out (reg) _);
                }
                BLUE_STACK_POINTER.borrow(cs).set(stack_prt);
                if led_state == 1 {
                    gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                    led_state = 0;
                } else {
                    gpiod.odr.modify(|_, w| w.odr15().set_bit());
                    led_state = 1;
                }
                unsafe {
                    asm ! ("MRS {1}, MSP\nMSR MSP, {2}\nPUSH {{{0}}}\nMRS {2}, MSP\nMSR MSP, {1}" , in (reg) led_state , out (reg) _ , in (reg) stack_prt);
                }
                BLUE_STACK_POINTER.borrow(cs).set(stack_prt);
            }
        });
        cortex_m::asm::delay(8000000);
    }
}
fn FlashRed() {
    loop {
        free(|cs| {
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                let mut stack_prt = RED_STACK_POINTER.borrow(cs).get();
                let mut led_state: u32 = 0;
                unsafe {
                    asm ! ("MRS {2}, MSP\nMSR MSP, {0}\nPOP {{{1}}}\nMRS {0}, MSP\nMSR MSP, {2}" , inout (reg) stack_prt , out (reg) led_state , out (reg) _);
                }
                RED_STACK_POINTER.borrow(cs).set(stack_prt);
                if led_state == 1 {
                    gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                    led_state = 0;
                } else {
                    gpiod.odr.modify(|_, w| w.odr14().set_bit());
                    led_state = 1;
                }
                unsafe {
                    asm ! ("MRS {1}, MSP\nMSR MSP, {2}\nPUSH {{{0}}}\nMRS {2}, MSP\nMSR MSP, {1}" , in (reg) led_state , out (reg) _ , in (reg) stack_prt);
                }
                RED_STACK_POINTER.borrow(cs).set(stack_prt);
            }
        });
        cortex_m::asm::delay(8000000);
    }
}
#[doc(hidden)]
#[export_name = "main"]
pub unsafe extern "C" fn __cortex_m_rt_main_trampoline() {
    __cortex_m_rt_main()
}
fn __cortex_m_rt_main() -> ! {
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr.write(|w| w.gpioden().set_bit());
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());
    rcc.apb1enr.write(|w| w.tim3en().set_bit());
    let tim3 = &stm32f4_peripherals.TIM3;
    tim3.psc.write(|w| w.psc().bits(15999));
    tim3.arr.write(|w| w.arr().bits(10));
    tim3.dier.write(|w| w.uie().set_bit());
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());
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
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);
    cortex_m::interrupt::free(|cs| {
        let mut led_state: u32 = 0x0;
        let mut blue_stack_prt: u32 = BLUE_STACK_POINTER.borrow(cs).get();
        let mut red_stack_prt: u32 = RED_STACK_POINTER.borrow(cs).get();
        let blue_status = 1;
        let red_status = 0;
        let blue_pc = 0x80009b0;
        let red_pc = 0x8000ba6;
        unsafe {
            asm ! ("MRS {3}, MSP\nMSR MSP, {0}\nPUSH {{{6}}}\nPUSH {{{4}}}\nPUSH {{{2}}}\nMRS {0}, MSP\nMSR MSP, {1}\nPUSH {{{7}}}\nPUSH {{{5}}}\nPUSH {{{2}}}\nMRS {1}, MSP\nMSR MSP, {3}" , inout (reg) blue_stack_prt , inout (reg) red_stack_prt , in (reg) led_state , out (reg) _ , in (reg) blue_status , in (reg) red_status , in (reg) blue_pc , in (reg) red_pc);
        }
        BLUE_STACK_POINTER.borrow(cs).set(blue_stack_prt);
        RED_STACK_POINTER.borrow(cs).set(red_stack_prt);
    });
    tim3.cr1.write(|w| w.cen().set_bit());
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
    });
    FlashBlue();
    FlashRed();
    loop {}
}
#[doc(hidden)]
#[export_name = "TIM3"]
pub unsafe extern "C" fn __cortex_m_rt_TIM3_trampoline() {
    __cortex_m_rt_TIM3()
}
fn __cortex_m_rt_TIM3() {
    {
        extern crate cortex_m_rt;
        interrupt::TIM3;
    }
    free(|cs| {
        if let (&mut Some(ref mut gpiod), &mut Some(ref mut tim3)) = (
            MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim3.sr.write(|w| w.uif().clear_bit());
            let mut blue_stack_prt: u32 = BLUE_STACK_POINTER.borrow(cs).get();
            let mut red_stack_prt: u32 = RED_STACK_POINTER.borrow(cs).get();
            let mut blue_led_state = 1;
            let mut blue_status = 0;
            let mut blue_pc = 0;
            let mut red_led_state = 1;
            let mut red_status = 0;
            let mut red_pc = 0;
            unsafe {
                asm ! ("MRS {4}, MSP\nMSR MSP, {0}\nPOP {{{2}}}\nPOP {{{5}}}\nPOP {{{7}}}\nMRS {0}, MSP\nMSR MSP, {1}\nPOP {{{3}}}\nPOP {{{6}}}\nPOP {{{8}}}\nMRS {1}, MSP\nMSR MSP, {4}" , inout (reg) blue_stack_prt , inout (reg) red_stack_prt , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _);
            }
            let mut exit_pc: u32;
            if red_status == 0 {
                blue_status = 0;
                red_status = 1;
                exit_pc = red_pc;
            } else if blue_status == 0 {
                red_status = 0;
                blue_status = 1;
                exit_pc = blue_pc;
            } else {
                exit_pc = blue_pc;
            }
            let mut pc = 0;
            unsafe {
                asm ! ("POP {{{0}}}\nPOP {{{1}}}\nPOP {{{2}}}\nPOP {{{3}}}\nPOP {{{4}}}\nPOP {{{5}}}\nPOP {{{6}}}\nPUSH {{{7}}}\nPUSH {{{5}}}\nPUSH {{{4}}}\nPUSH {{{3}}}\nPUSH {{{2}}}\nPUSH {{{1}}}\nPUSH {{{0}}}" , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) _ , out (reg) pc , in (reg) exit_pc);
            }
            if red_status == 0 {
                blue_pc = pc;
            } else if blue_status == 0 {
                red_pc = pc;
            }
            unsafe {
                asm ! ("MRS {4}, MSP\nMSR MSP, {0}\nPUSH {{{7}}}\nPUSH {{{5}}}\nPUSH {{{2}}}\nMRS {0}, MSP\nMSR MSP, {1}\nPUSH {{{8}}}\nPUSH {{{6}}}\nPUSH {{{3}}}\nMRS {1}, MSP\nMSR MSP, {4}" , inout (reg) blue_stack_prt , inout (reg) red_stack_prt , in (reg) blue_led_state , in (reg) red_led_state , out (reg) _ , in (reg) blue_status , in (reg) red_status , in (reg) blue_pc , in (reg) red_pc);
            }
        }
    });
}
