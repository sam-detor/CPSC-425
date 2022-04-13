#![no_std]
#![no_main]


extern crate cortex_m;
extern crate cortex_m_rt as rt;
use panic_halt as _;
extern crate stm32f4;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f411::{self};

use core::cell::RefCell;
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;

static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Getting access to the peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // Enabling GPIOD and SYSCFG clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr.write(|w| w.gpioden().set_bit());
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());

    //Enable tim3 clocks
    rcc.apb1enr.write(|w| w.tim3en().set_bit());

    //get access to timer
    let tim3 = &stm32f4_peripherals.TIM3;

    //set prescalar values
    //to turn an 8mHz clock into 1ms intervals
    tim3.psc.write(|w| w.psc().bits(15999));

    //set auto refil values (10ms)
    tim3.arr.write(|w| w.arr().bits(10));

    //enable interrupts
    tim3.dier.write(|w| w.uie().set_bit());

    //set as a repetitive timer
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    // Set blue led pins to output
    let gpiod = &stm32f4_peripherals.GPIOD;
    gpiod
        .moder
        .write(|w| w.moder15().output());

    //Moving GPIOD into mutex
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
    
    //enabling the timer
    tim3.cr1.write(|w| w.cen().set_bit());

    //moving the timer into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
    });

    flash_blue();
    loop {
       
    }
}

#[interrupt]

fn TIM3() {
    //triggeres every 10ms, just returns

    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim3) = 
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut()
         {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

        }});
}

fn flash_blue() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = 
                MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut()
            {
                //update led based on led_state
                if led_state == 1 {
                    gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                    led_state = 0;
                } else {
                    gpiod.odr.modify(|_, w| w.odr15().set_bit());
                    led_state = 1;
                }
            }
        });
        
        cortex_m::asm::delay(8000000);
    }
}
