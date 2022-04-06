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

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;

static MUTEX_GPIOC: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOC>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_EXTI: Mutex<RefCell<Option<stm32f4::stm32f411::EXTI>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM4: Mutex<RefCell<Option<stm32f4::stm32f411::TIM4>>> =
    Mutex::new(RefCell::new(None));

static PLAYING: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static MY_COLOR: Mutex<Cell<u32>> = Mutex::new(Cell::new(1));

#[entry]
fn main() -> ! {
    // Getting access to the peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // Enabling GPIOC, GPIOD and SYSCFG clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr
        .write(|w| w.gpiocen().set_bit().gpioden().set_bit());
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());

    //Enable tim3 and tim4 clocks
    rcc.apb1enr
        .write(|w| w.tim3en().set_bit().tim4en().set_bit());
    
    //get access to timers
    let tim3 = &stm32f4_peripherals.TIM3;
    let tim4 = &stm32f4_peripherals.TIM4;

    //set prescalar values
    //to turn an 8mHz clock into 1ms intervals
    tim3.psc.write(|w| w.psc().bits(15999));
    tim4.psc.write(|w| w.psc().bits(15999));

    //set auto refil values
    tim3.arr.write(|w| w.arr().bits(500));
    tim4.arr.write(|w| w.arr().bits(100));

    //enable interrupts
    tim3.dier.write(|w| w.uie().set_bit());
    tim4.dier.write(|w| w.uie().set_bit());

    //set as a repetitive timer
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());
    tim4.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    // 3. Configure PC7 pin as input, floating
    let gpioc = &stm32f4_peripherals.GPIOC;
    gpioc.moder.write(|w| w.moder7().input());
    gpioc.pupdr.write(|w| w.pupdr7().floating());

    // Set led pins to output
    let gpiod = &stm32f4_peripherals.GPIOD;
    gpiod.moder.write(|w| {
        w.moder12()
            .output()
            .moder13()
            .output()
            .moder14()
            .output()
            .moder15()
            .output()
    });

    // 4. connect EXTI7 line to PC7 pin
    let syscfg = &stm32f4_peripherals.SYSCFG;
    syscfg.exticr2.write(|w| unsafe { w.exti7().bits(0b0010) });

    let exti = &stm32f4_peripherals.EXTI;
    exti.imr.write(|w| w.mr7().set_bit()); // unmask interrupt
    exti.ftsr.write(|w| w.tr7().set_bit()); // falling edge trigger

    // Move shared peripherals into mutexes
    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOC
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOC));
        MUTEX_EXTI
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.EXTI));
        MUTEX_GPIOD
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOD));
    });

    // 7. Enable EXTI7 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 2);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);

        nvic.set_priority(Interrupt::TIM4, 3);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM4);

        nvic.set_priority(Interrupt::EXTI9_5, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI9_5);
    }

    cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI9_5);
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM4);

    //enabling the timers
    tim3.cr1.write(|w| w.cen().set_bit());
    tim4.cr1.write(|w| w.cen().set_bit());

    //moving the timers into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
        MUTEX_TIM4
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM4));
    });


    loop {
    }
}



#[interrupt]
fn EXTI9_5() { //triggered when PC7 is touched
    
    static mut STATUS: i32 = 0;
    
    cortex_m::interrupt::free(|cs| {
        let refcell = MUTEX_EXTI.borrow(cs).borrow();
        let exti = match refcell.as_ref() {
            None => return,
            Some(v) => v,
        };
        // clear the EXTI line 7 pending bit
        exti.pr.write(|w| w.pr7().set_bit());

        if *STATUS == 0 { //if PC7 was touched for the first timer
            PLAYING.borrow(cs).set(false); //pause leds
            *STATUS += 1;
        } else { //if PC7 was touched again
            PLAYING.borrow(cs).set(true); //restart sequence from green
            MY_COLOR.borrow(cs).set(1);
            *STATUS = 0;
        }
    });
}

#[interrupt]

fn TIM4() { //triggered every 0.1s, refreshes the pin
    free(|cs| {

        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut gpioc), &mut Some(ref mut tim4)) = (
            MUTEX_GPIOC.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM4.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim4.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            gpioc.moder.write(|w| w.moder7().output());
            gpioc.odr.modify(|_, w| w.odr7().set_bit()); //charges PC7 to high
            gpioc.moder.write(|w| w.moder7().input());
        }
    });
}

#[interrupt]

fn TIM3() {//triggeres every 0.5s, blinks leds based on PLAYING and MY_COLOR
    
    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut gpiod), &mut Some(ref mut tim3)) = (
            MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            let playing = PLAYING.borrow(cs).get();
            let mut my_color = MY_COLOR.borrow(cs).get();

            if playing {
                if my_color == 1 {
                    gpiod.odr.write(|w| w.odr12().set_bit());// .odr15().clear_bit());
                } else if my_color == 2 {
                    gpiod.odr.write(|w| w.odr13().set_bit()); //.odr12().clear_bit());
                } else if my_color == 3 {
                    gpiod.odr.write(|w| w.odr14().set_bit()); //.odr13().clear_bit());
                } else{
                    gpiod.odr.write(|w| w.odr15().set_bit());//..odr14().clear_bit());
                }

                if my_color == 4 {
                    my_color = 1;
                } else {
                    my_color += 1;
                }
            }
            MY_COLOR.borrow(cs).set(my_color);
        }
    });
}



