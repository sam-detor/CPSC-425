#![no_std]
#![no_main]

// import all the necessary crates and components

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f4;
extern crate stm32f4xx_hal as hal;

use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;

use crate::hal::{
    delay::Delay,
    gpio::{
        gpioa::{PA0},
        Edge, ExtiPin, Input, Floating,
    },
    prelude::*,
    stm32,
    stm32::{Interrupt, EXTI},
    time::Hertz,
    timer::{Event, Timer},
};

// create two globally accessible values for set and elapsed time
static PLAYING: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static MY_COLOR: Mutex<Cell<u32>> = Mutex::new(Cell::new(1));

// globally accessible interrupts and peripherals: timer, external interrupt and button
static TIMER_TIM2: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));
static EXTI: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));
static BUTTON: Mutex<RefCell<Option<PA0<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let gpiod = dp.GPIOD.split();
        let gpioa = dp.GPIOA.split();
        // necessary to enable this for the external interrupt to work
        dp.RCC.apb2enr.write(|w| w.syscfgen().enabled());

        // set up clocks
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

        let mut red_led = gpiod.pd14.into_push_pull_output();
        let mut blue_led = gpiod.pd15.into_push_pull_output();
        let mut green_led = gpiod.pd12.into_push_pull_output();
        let mut orange_led = gpiod.pd13.into_push_pull_output();

        //set up the on-board button on PA0
        let mut board_btn = gpioa.pa0.into_floating_input();
        board_btn.make_interrupt_source(&mut dp.SYSCFG);
        board_btn.enable_interrupt(&mut dp.EXTI);
        board_btn.trigger_on_edge(&mut dp.EXTI, Edge::RISING_FALLING);

        // set up delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // set up timers and external interrupt
        let mut timer = Timer::tim2(dp.TIM2, Hertz(1), clocks);
        timer.listen(Event::TimeOut);

        let exti = dp.EXTI;

        free(|cs| {
            TIMER_TIM2.borrow(cs).replace(Some(timer));
            EXTI.borrow(cs).replace(Some(exti));
            BUTTON.borrow(cs).replace(Some(board_btn));
        });

        let mut nvic = cp.NVIC;
        unsafe {
            nvic.set_priority(Interrupt::TIM2, 2);
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);

            nvic.set_priority(Interrupt::EXTI0, 1);
            cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0);
        }

        cortex_m::peripheral::NVIC::unpend(Interrupt::TIM2);
        cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI0);

        // set the counter to some value, in this case 3 minutes
        // count down as long as the value > 0

        loop {
            free(|cs| {
                // Obtain all Mutex protected resources
                let playing = PLAYING.borrow(cs).get();
                let mut my_color = MY_COLOR.borrow(cs).get();

                if playing {
                    if my_color == 1 {
                        green_led.set_high().unwrap();
                        blue_led.set_low().unwrap();
                    } else if my_color == 2 {
                        orange_led.set_high().unwrap();
                        green_led.set_low().unwrap();
                    } else if my_color == 3 {
                        red_led.set_high().unwrap();
                        orange_led.set_low().unwrap();
                    } else if my_color == 4 {
                        blue_led.set_high().unwrap();
                        red_led.set_low().unwrap();
                    }
                    else {
                        blue_led.set_low().unwrap();
                        green_led.set_high().unwrap();
                        red_led.set_low().unwrap();
                        orange_led.set_low().unwrap();
                        my_color = 1;
                    }

                    if my_color == 4 {
                        my_color = 1;
                    } else {
                        my_color += 1;
                    }
                }
                MY_COLOR.borrow(cs).set(my_color);
            });
            delay.delay_ms(500_u16);
        }
    }

    loop {}
}

#[interrupt]

// the ELAPSED value gets updated every second when the interrupt fires

fn TIM2() {
    // enter critical section
    //loop {}
    free(|cs| {
        stm32::NVIC::unpend(Interrupt::TIM2);
        if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }

        // decrease the ELAPSED value by 1 second
    });
}

#[interrupt]

fn EXTI0() {
    // Enter critical section
    //loop {}
    static mut STATUS: i32 = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut btn), &mut Some(ref mut exti)) = (
            BUTTON.borrow(cs).borrow_mut().deref_mut(),
            EXTI.borrow(cs).borrow_mut().deref_mut(),
        ) {
            btn.clear_interrupt_pending_bit(exti);
        
            if *STATUS == 0 {
                PLAYING.borrow(cs).set(false);
                *STATUS += 1;
            } else {
                PLAYING.borrow(cs).set(true);
                MY_COLOR.borrow(cs).set(5);
                *STATUS = 0;
            }
        
        }
    });

}
