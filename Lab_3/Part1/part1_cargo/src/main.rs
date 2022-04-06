#![no_std]
#![no_main]

// import all the necessary crates and components

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate stm32f4;
extern crate stm32f4xx_hal as hal;

use panic_halt as _;

use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;

use crate::hal::{
    delay::Delay,
    gpio::{gpioa::PA0, Edge, ExtiPin, Floating, Input},
    prelude::*,
    stm32,
    stm32::{Interrupt, EXTI}
};

// create two globally accessible values for the paused/playing state and the LED state
static PLAYING: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static MY_COLOR: Mutex<Cell<u32>> = Mutex::new(Cell::new(1));

// globally accessible interrupts and peripherals: external interrupt and button
static EXTI: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));
static BUTTON: Mutex<RefCell<Option<PA0<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        //Get Access to the GPIOs
        let gpiod = dp.GPIOD.split();
        let gpioa = dp.GPIOA.split();

        // necessary to enable this for the external interrupt to work
        dp.RCC.apb2enr.write(|w| w.syscfgen().enabled());

        // set up clocks
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();

        //configure leds
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

        let exti = dp.EXTI;

        //move peripherals into mutexs
        free(|cs| {
            EXTI.borrow(cs).replace(Some(exti));
            BUTTON.borrow(cs).replace(Some(board_btn));
        });

        //enable + set priority of interrupts
        let mut nvic = cp.NVIC;
        unsafe {
            nvic.set_priority(Interrupt::EXTI0, 1);
            cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0);
        }

        cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI0);

        //blink the leds according to the state of playing and my color

        loop {
            free(|cs| {
                // Obtain all Mutex protected resources
                let playing = PLAYING.borrow(cs).get();
                let mut my_color = MY_COLOR.borrow(cs).get();

                if playing {
                    if my_color == 1 { //set green
                        green_led.set_high().unwrap();
                        blue_led.set_low().unwrap();
                    } else if my_color == 2 { //set orange
                        orange_led.set_high().unwrap();
                        green_led.set_low().unwrap();
                    } else if my_color == 3 { //set red
                        red_led.set_high().unwrap();
                        orange_led.set_low().unwrap();
                    } else if my_color == 4 { //set blue
                        blue_led.set_high().unwrap();
                        red_led.set_low().unwrap();
                    } else {//after interrupt turn off all leds, set green, state is now 1
                        blue_led.set_low().unwrap();
                        green_led.set_high().unwrap();
                        red_led.set_low().unwrap();
                        orange_led.set_low().unwrap();
                        my_color = 1;
                    }

                    if my_color == 4 { //update my_color
                        my_color = 1;
                    } else {
                        my_color += 1;
                    }
                }
                MY_COLOR.borrow(cs).set(my_color); //store the new value of my color
            });
            delay.delay_ms(500_u16); //delay for 0.5s
        }
    }

    loop {}
}

#[interrupt]
fn EXTI0() { //triggered when button is pressed or unpressed
    static mut STATUS: i32 = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut btn), &mut Some(ref mut exti)) = (
            BUTTON.borrow(cs).borrow_mut().deref_mut(),
            EXTI.borrow(cs).borrow_mut().deref_mut(),
        ) {
            btn.clear_interrupt_pending_bit(exti); //clear interrupt

            if *STATUS == 0 { //if the button is pressed
                PLAYING.borrow(cs).set(false); //pause sequence
                *STATUS += 1;
            } else { //if the button is being unpressed
                PLAYING.borrow(cs).set(true); //restart sequence from green
                MY_COLOR.borrow(cs).set(5);
                *STATUS = 0;
            }
        }
    });
}
