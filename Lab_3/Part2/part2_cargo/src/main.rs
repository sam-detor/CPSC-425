// This code is part of an article on using DMA with Embedded Rust
//
// See: https://flowdsp.io/blog/stm32f3-01-interrupts/

#![no_std]
#![no_main]

#[allow(unused_imports)]
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f4;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f411::{self};

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use stm32f4::stm32f411::interrupt;
use stm32f4::stm32f411::Interrupt;

static MUTEX_GPIOC: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOC>>> = Mutex::new(RefCell::new(None));
static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));
static MUTEX_EXTI: Mutex<RefCell<Option<stm32f4::stm32f411::EXTI>>> =
    Mutex::new(RefCell::new(None));
//static MUTEX_TIM2:  Mutex<RefCell<Option<stm32f4::stm32f411::TIM2>>>  = Mutex::new(RefCell::new(None));
static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));
static MUTEX_TIM4: Mutex<RefCell<Option<stm32f4::stm32f411::TIM4>>> =
    Mutex::new(RefCell::new(None));

static PLAYING: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static MY_COLOR: Mutex<Cell<u32>> = Mutex::new(Cell::new(1));

static mut GPIOC: Option<stm32f4::stm32f411::GPIOC> = None;

#[entry]
fn main() -> ! {
    //set_sysclk_to_100();
    // 1. get peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // 2. enable GPIOA and SYSCFG clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr
        .write(|w| w.gpiocen().set_bit().gpioden().set_bit());
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());

    //timer 2 and 3 enable
    rcc.apb1enr
        .write(|w| w.tim3en().set_bit().tim4en().set_bit());

    let tim3 = &stm32f4_peripherals.TIM3;
    let tim4 = &stm32f4_peripherals.TIM4;

    //prescalar values
    tim3.psc.write(|w| w.psc().bits(15999));
    tim4.psc.write(|w| w.psc().bits(15999));

    //auto refil values
    tim3.arr.write(|w| w.arr().bits(500));
    tim4.arr.write(|w| w.arr().bits(100));

    tim3.dier.write(|w| w.uie().set_bit());
    tim4.dier.write(|w| w.uie().set_bit());

    //non one pulse
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());
    tim4.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    // 3. Configure PC7 pin as input, pull-down
    let gpioc = &stm32f4_peripherals.GPIOC;
    gpioc.moder.write(|w| w.moder7().input());
    gpioc.pupdr.write(|w| w.pupdr7().floating());

    // configure PE8, PE9 as output
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

    // 4. connect EXTI0 line to PA0 pin
    let syscfg = &stm32f4_peripherals.SYSCFG;
    syscfg.exticr2.write(|w| unsafe { w.exti7().bits(0b0010) });

    let exti = &stm32f4_peripherals.EXTI;
    exti.imr.write(|w| w.mr7().set_bit()); // unmask interrupt
    exti.ftsr.write(|w| w.tr7().set_bit()); // trigger=rising-edge

    // 6. Move shared peripherals into mutexes
    //    After this we can only access them via their respective mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOC.borrow(cs).replace(Some(stm32f4_peripherals.GPIOC));
        MUTEX_EXTI
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.EXTI));
        MUTEX_GPIOD
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOD));
    });

    // 7. Enable EXTI0 Interrupt
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

    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
        MUTEX_TIM4
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM4));
    });

    //unsafe {
        //GPIOC = Some(stm32f4_peripherals.GPIOC);
    //}

    loop {

        //gpiod.odr.modify(|_, w| w.odr12().set_bit());
        /*
            free(|cs| {
                // Obtain all Mutex protected resources
                if let &mut Some(ref mut gpiod) = (MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut())
                {
                    gpiod.odr.modify(|_, w| w.odr14().set_bit());
                }
            });
            cortex_m::asm::delay(2000000);
            free(|cs| {
                // Obtain all Mutex protected resources
                if let &mut Some(ref mut gpiod) = (MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut())
                {
                    gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                }
            });
            cortex_m::asm::delay(2000000);
        */
    }
}

// 8. Handle interrupt

#[interrupt]
fn EXTI9_5() {
    static mut STATUS: i32 = 0;
    // clear the EXTI line 0 pending bit
    cortex_m::interrupt::free(|cs| {
        let refcell = MUTEX_EXTI.borrow(cs).borrow();
        let exti = match refcell.as_ref() {
            None => return,
            Some(v) => v,
        };
        exti.pr.write(|w| w.pr7().set_bit());
    });

    // toggle LED4
    cortex_m::interrupt::free(|cs| {
        if *STATUS == 0 {
            PLAYING.borrow(cs).set(false);
            *STATUS += 1;
        } else {
            PLAYING.borrow(cs).set(true);
            MY_COLOR.borrow(cs).set(5);
            *STATUS = 0;
        }
    });
}

#[interrupt]

fn TIM4() {
    //loop {}
    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut gpioc), &mut Some(ref mut tim4) ) =
            (MUTEX_GPIOC.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM4.borrow(cs).borrow_mut().deref_mut())
        {
            tim4.sr.write(|w| w.uif().clear_bit());

            gpioc.moder.write(|w| w.moder7().output());
            gpioc.odr.modify(|_, w| w.odr7().set_bit());
            gpioc.moder.write(|w| w.moder7().input());
        }     
});
/* 
    unsafe {
        let gpioc_new = gpioc_getter();
        
    }
    */
}

#[interrupt]

fn TIM3() {
    //loop {}
    static mut STATUS: bool = true;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut gpiod), &mut Some(ref mut tim3)) = (
            MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut(),
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim3.sr.write(|w| w.uif().clear_bit());

            let playing = PLAYING.borrow(cs).get();
            let mut my_color = MY_COLOR.borrow(cs).get();
            //gpiod.odr.modify(|_, w| w.odr12().set_bit());

            if playing {
                if my_color == 1 {
                    gpiod.odr.write(|w| w.odr12().set_bit().odr15().clear_bit());
                } else if my_color == 2 {
                    gpiod.odr.write(|w| w.odr13().set_bit().odr12().clear_bit());
                } else if my_color == 3 {
                    gpiod.odr.write(|w| w.odr14().set_bit().odr13().clear_bit());
                } else if my_color == 4 {
                    gpiod.odr.write(|w| w.odr15().set_bit().odr14().clear_bit());
                } else {
                    gpiod.odr.write(|w| {
                        w.odr12()
                            .set_bit()
                            .odr15()
                            .clear_bit()
                            .odr13()
                            .clear_bit()
                            .odr14()
                            .clear_bit()
                    });
                    my_color = 1;
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

fn set_sysclk_to_100() {
    let peripherals = stm32f411::Peripherals::take().unwrap();
    let rcc = &peripherals.RCC;
    let pwr = &peripherals.PWR;
    let flash = &peripherals.FLASH;

    let PLL_M = 8;
    let PLL_Q = 7;
    let PLL_P = 4;
    let PLL_N = 400;

    rcc.cr.write(|w| w.hseon().set_bit());
    /* Enable HSE (CR: bit 16) */
    //RCC->CR |= (1U << 16);
    /* Wait till HSE is ready (CR: bit 17) */
    //while(!(RCC->CR & (1 << 17)));
    /*
    while !rcc.cr.read().hserdy().bit_is_set() {

    };
    */
    /* Enable power interface clock (APB1ENR:bit 28) */
    //RCC->APB1ENR |= (1 << 28);
    rcc.apb1enr.write(|w| w.pwren().set_bit());

    /* set voltage scale to 1 for max frequency (PWR_CR:bit 14)
     * (0b0) scale 2 for fCLK <= 144 Mhz
     * (0b1) scale 1 for 144 Mhz < fCLK <= 168 Mhz
     */
    //PWR->CR |= (1 << 14);
    pwr.cr.write(|w| unsafe { w.vos().bits(0b11) });

    /* set AHB prescaler to /1 (CFGR:bits 7:4) */
    //RCC->CFGR |= (0 << 4);
    rcc.cfgr.write(|w| w.hpre().div1()); //  bits(0b0000)});
                                         /* set APB low speed prescaler to /4 (APB1) (CFGR:bits 12:10) */
    //RCC->CFGR |= (5 << 10);
    rcc.cfgr.write(|w| w.ppre1().div4()); //   bits(0b101)});
                                          /* set APB high speed prescaler to /2 (APB2) (CFGR:bits 15:13) */
    //RCC->CFGR |= (4 << 13);
    rcc.cfgr.write(|w| w.ppre2().div2()); //   bits(0b100)});

    /* Set M, N, P and Q PLL dividers
     * PLLCFGR: bits 5:0 (M), 14:6 (N), 17:16 (P), 27:24 (Q)
     * Set PLL source to HSE, PLLCFGR: bit 22, 1:HSE, 0:HSI
     */
    //RCC->PLLCFGR = PLL_M | (PLL_N << 6) | (((PLL_P >> 1) -1) << 16) |
    //(PLL_Q << 24) | (1 << 22);

    rcc.pllcfgr.write(|w| unsafe { w.pllm().bits(PLL_M) });
    rcc.pllcfgr.write(|w| unsafe { w.plln().bits(PLL_N) });
    rcc.pllcfgr.write(|w| unsafe { w.pllq().bits(PLL_Q) });
    rcc.pllcfgr.write(|w| w.pllp().bits(PLL_P));
    rcc.pllcfgr.write(|w| w.pllsrc().set_bit());

    rcc.cr.write(|w| w.pllon().set_bit());
    /* Enable the main PLL (CR: bit 24) */
    //RCC->CR |= (1 << 24);
    /* Wait till the main PLL is ready (CR: bit 25) */
    //while(!(RCC->CR & (1 << 25)));
    /*
    while !rcc.cr.read().plli2srdy().bit_is_set() {

    };
    */
    /* Configure Flash
     * prefetch enable (ACR:bit 8)
     * instruction cache enable (ACR:bit 9)
     * data cache enable (ACR:bit 10)
     * set latency to 5 wait states (ARC:bits 2:0)
     *   see Table 10 on page 80 in RM0090
     */
    //FLASH->ACR = (1 << 8) | (1 << 9) | (1 << 10 ) | (5 << 0);
    flash
        .acr
        .write(|w| w.dcen().set_bit().icen().set_bit().prften().set_bit());

    flash.acr.write(|w| unsafe { w.latency().bits(5) });
    /* Select the main PLL as system clock source, (CFGR:bits 1:0)
     * 0b00 - HSI
     * 0b01 - HSE
     * 0b10 - PLL
     */
    rcc.cfgr.write(|w| w.sw().pll()); //   bits(0b10)});
                                      //RCC->CFGR &= ~(3U << 0);
                                      //RCC->CFGR |= (2 << 0);
                                      /* Wait till the main PLL is used as system clock source (CFGR:bits 3:2) */
    /*
    while !rcc.cfgr.read().sws().is_pll() {

    };
    */
}

unsafe fn gpioc_getter() -> &'static mut stm32f4::stm32f411::GPIOC {
    match GPIOC {
        Some(ref mut x) => &mut *x,
        None => panic!(),
    }
}
