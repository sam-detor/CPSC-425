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

static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

const STACK_SIZE: u32 = 2048; //2KB
const STACK_BASE: u32 = 0x20000900;
const STACK_MAX: u32 = 0x2001F700;
//const HEAP_SIZE: usize = 0x200;
//const HEAP_START: usize = 0x20000300;
const NUM_TASKS: usize = 4;

//static NUM_TASKS: usize = 4;
//static NUM_TASKS: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static TASK_TIC_COUNTER: Mutex<RefCell<[u32;NUM_TASKS]>> = Mutex::new(RefCell::new([0;NUM_TASKS]));
//static TASK_FUNC_POINTERS: Mutex<RefCell<Vec<u32>>> = Mutex::new(RefCell::new(Vec::new()));
static mut TASK_STACK_POINTERS: [u32;NUM_TASKS] = [0;NUM_TASKS];
static TASK_RUNNING: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
//static TASK_POINTERS: [u32; NUM_TASKS] = [FlashRed as u32, FlashBlue as u32, FlashOrange as u32, FlashGreen as u32];

//static mut TASK_RUNNING: u32 = 0; // Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
//static mut BLUE_STACK_POINTER: u32 = 0x20000300;
//static mut RED_STACK_POINTER: u32 = 0x20000700;

fn FlashBlue() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                    //100 milisec in a sec
                    //led state stuff
                    if led_state == 1 {
                        gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                        led_state = 0;
                    } else {
                        gpiod.odr.modify(|_, w| w.odr15().set_bit());
                        led_state = 1;
                    }
                }
        });
        //delay_test();
        delay(100);//, 0);
    }
}

fn FlashRed() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                    //100 milisec in a sec
                    //led state stuff
                    if led_state == 1 {
                        gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                        led_state = 0;
                    } else {
                        gpiod.odr.modify(|_, w| w.odr14().set_bit());
                        led_state = 1;
                    }
                }
        });
        //delay_test();
        delay(50);//, 1);
    }
}

fn FlashOrange() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                    //100 milisec in a sec
                    //led state stuff
                    if led_state == 1 {
                        gpiod.odr.modify(|_, w| w.odr13().clear_bit());
                        led_state = 0;
                    } else {
                        gpiod.odr.modify(|_, w| w.odr13().set_bit());
                        led_state = 1;
                    }
                }
        });
        //delay_test();
        delay(25);//, 2);
    }
}

fn FlashGreen() {
    let mut led_state = 0;
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
                    //100 milisec in a sec
                    //led state stuff
                    if led_state == 1 {
                        gpiod.odr.modify(|_, w| w.odr12().clear_bit());
                        led_state = 0;
                    } else {
                        gpiod.odr.modify(|_, w| w.odr12().set_bit());
                        led_state = 1;
                    }
                }
        });
        //delay_test();
        delay(200);//, 3);
    }
}

#[entry]
fn main() -> ! {

    // Getting access to the peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // Enabling GPIOC clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr.write(|w| w.gpioden().set_bit());
    
    //enable syscfg clock
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());
    
    //Enable tim3 clock
    rcc.apb1enr.write(|w| w.tim3en().set_bit());

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

    // 7. Enable EXTI7 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

    //moving the timers into the mutex
    cortex_m::interrupt::free(|cs| {
    MUTEX_TIM3
        .borrow(cs)
        .replace(Some(stm32f4_peripherals.TIM3));
    });

    //init_scheduler();
    initialize_tasks([FlashBlue as u32, FlashRed as u32, FlashOrange as u32, FlashGreen as u32]);
    start_scheduler();

    loop {}
}
//r0 - r3
#[interrupt]
fn TIM3() {
    //triggeres every 0.5s, blinks leds based on PLAYING and MY_COLOR 
    static mut WHOSE_RUNNING: usize = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut tim3), &mut ref mut task_tic) = 
            (MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(), 
            TASK_TIC_COUNTER.borrow(cs).borrow_mut().deref_mut())
        {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit
        
            for i in 0..NUM_TASKS {
                task_tic[i] += 1;
            }
            let prev_task = TASK_RUNNING.borrow(cs).get();
            if prev_task == NUM_TASKS {
                TASK_RUNNING.borrow(cs).set(1);
            }
            else{
                TASK_RUNNING.borrow(cs).set(prev_task + 1);
            }
        }});    
            
        unsafe{
            
            if *WHOSE_RUNNING == 0 {
                *WHOSE_RUNNING = 1;
                context_switch_orig(TASK_STACK_POINTERS[0], &TASK_STACK_POINTERS[0]);
                
            }
            else if *WHOSE_RUNNING == NUM_TASKS
            {
                let index = *WHOSE_RUNNING - 1;
                *WHOSE_RUNNING = 1;
                context_switch(&TASK_STACK_POINTERS[index], TASK_STACK_POINTERS[0], &TASK_STACK_POINTERS[0]); 

            }
            else {
                *WHOSE_RUNNING  = *WHOSE_RUNNING + 1;
                context_switch(&TASK_STACK_POINTERS[*WHOSE_RUNNING - 2], TASK_STACK_POINTERS[*WHOSE_RUNNING - 1], &TASK_STACK_POINTERS[*WHOSE_RUNNING - 1]); 
            }
            
        }  
        }

#[naked]
#[no_mangle]
  pub unsafe extern "C" fn context_switch(old_addy: &u32, new_ptr: u32, new_addy: &u32) { //pointer in r0, pc in r1
    asm! (
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
  pub unsafe extern "C" fn context_switch_orig(ptr: u32, prt_addy: &u32) { //pointer in r0, pc in r1
    asm! (
        "MSR MSP, r0",
        "POP {{r4-r11}}",
        "POP {{LR}}",
        "STR SP, [r1]",
        "blx LR", options(noreturn),
    );
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn trampoline() {
    asm!(
        "ldr lr, =0xfffffff9",
        "bx lr", options(noreturn),
    );
}

fn initialize_tasks (func_ptrs: [u32; NUM_TASKS]) -> u8 {
    //let mut retVal = 0;
        // Obtain all Mutex protected resources
    for task_id in 0..NUM_TASKS { 
        let mut stack_prt = STACK_BASE + ((task_id as u32) * STACK_SIZE);
        if stack_prt > STACK_MAX {
            return 1;
        }
        stack_prt = create_stack(stack_prt, func_ptrs[task_id]);
        unsafe {
            TASK_STACK_POINTERS[task_id] = stack_prt;
        }
    }
    return 0;

    //create stack pointer
    //create stack
    //add function pointer to array
}

fn create_stack (sp: u32, func_ptr: u32) -> u32 {

    let mut my_sp = sp;
    let xpcr = 1 << 24;
    let dummy_val = 0;
    let all_regs_lr: u32 = trampoline as u32;
    unsafe {
        //set up blue stack
        asm!(
            "MRS {old_stack_prt}, MSP",
            "MSR MSP, {my_sp}",
            "PUSH {{{xpcr}}}",
            "PUSH {{{func_ptr}}}",
            "PUSH {{{func_ptr}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{dummy_val}}}",
            "PUSH {{{all_regs_lr}}}",
            "PUSH {{r4-r11}}",
            "MRS {my_sp}, MSP",
            "MSR MSP, {old_stack_prt}",
            my_sp = inout(reg) my_sp,
            xpcr = in(reg) xpcr,
            func_ptr = in(reg) func_ptr,
            old_stack_prt = out (reg) _,
            dummy_val = in(reg) dummy_val,
            all_regs_lr = in(reg) all_regs_lr,

        );
    }
    return my_sp;
}

/* 
fn init_scheduler() {
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();
    
    let rcc = &stm32f4_peripherals.RCC;
    
    //enable syscfg clock
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());
    
    //Enable tim3 clock
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

    // 7. Enable EXTI7 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

    //moving the timers into the mutex
    cortex_m::interrupt::free(|cs| {
    MUTEX_TIM3
        .borrow(cs)
        .replace(Some(stm32f4_peripherals.TIM3));
    });


}
*/
fn start_scheduler () {
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim3) = 
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut()
        {
             //enable the timer
            tim3.cr1.write(|w| w.cen().set_bit());
        }});
}

fn delay(delay_10ms: u32) {//}, task_id: usize) {
    //figure out what task
    /* 
    let main_sp = cortex_m::register::msp::read();
    let mut task_id = 0;
    let mut end_loop = false;
    unsafe {
        for i in 0..NUM_TASKS{
            if main_sp <= TASK_STACK_POINTERS[i] {
                break;
            }
            task_id += 1;
        } 
    }
    */
    let mut end_loop = false;
    let mut task_id: usize = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut ref mut task_tic = 
            TASK_TIC_COUNTER.borrow(cs).borrow_mut().deref_mut()
        {
             //enable the timer
            task_id = TASK_RUNNING.borrow(cs).get() - 1;
            task_tic[task_id] = 0;
        }});
    
    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut ref mut task_tic = 
                TASK_TIC_COUNTER.borrow(cs).borrow_mut().deref_mut()
            {
                if task_tic[task_id] >= delay_10ms
                {
                    end_loop = true;
                }
            }});
        if end_loop
        {
            break;
        }
    }
    //zero respective counter init local var to 0, check counter every cycle
}
/* 
#[no_mangle]
fn delay_test() {
    for i in 0..10000000 {

    }
}
*/