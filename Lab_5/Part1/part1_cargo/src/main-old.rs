#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(type_ascription)]

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

//const STACK_SIZE: u32 = 2048; //2KB
const STACK_BASE: u32 = 0x20000900;
const STACK_MAX: u32 = 0x2001F700;
const MAX_TASKS: usize = 2; //1 user tasks, 1 idle task

const GREEN: u32 = 1;
const ORANGE: u32 = 2;
const RED: u32 = 3;
const BLUE: u32 = 4;

struct TaskInfo {
    loaded: bool,
    awake: bool,
    stack_pointer: u32,
    task_tic: u32,
    stack_size: u32,
}

static mut TASK_INFO_ARRAY: [TaskInfo; MAX_TASKS] = [TaskInfo{loaded:true,awake:true,stack_pointer:0,task_tic:0, stack_size:0}, 
                                                      TaskInfo{loaded:false,awake:true,stack_pointer:0,task_tic:0,stack_size:0},];

static TASK_RUNNING: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));

extern "C" {
    fn start();
}

#[entry]
fn main() -> ! {
    // Getting access to the peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // Enabling GPIOD clock
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

    //move gpiod into mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOD
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOD));
    });

    //Setting up scheduler
    //get access to tim3
    let tim3 = &stm32f4_peripherals.TIM3;

    //set prescalar value
    //to turn an 8mHz clock into 1ms intervals
    tim3.psc.write(|w| w.psc().bits(15999));

    //set auto refil value
    tim3.arr.write(|w| w.arr().bits(10));

    //enable interrupt
    tim3.dier.write(|w| w.uie().set_bit());

    //set as a repetitive timer
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    // 7. Enable TIM3 Interrupt
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);
    }
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);

    //moving the tim3 into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
    });

    initialize_task(start as u32, 2048);
    start_scheduler();

    loop {}
}

#[interrupt]
fn TIM3() {
    //triggeres every 10ms and switches the running task

    static mut WHOSE_RUNNING: usize = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim3) = 
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut()
        {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            //add 1 to task timer
            unsafe{
            TASK_INFO_ARRAY[1].task_tic += 1;
            }

            //update global variable which task is running
            let prev_task = TASK_RUNNING.borrow(cs).get();
            let new_task: usize;
            if prev_task == MAX_TASKS - 1 {
                 new_task = TASK_RUNNING.borrow(cs).set(1);
            } else {
                TASK_RUNNING.borrow(cs).set(prev_task + 1);
            }
        }
    });

    //switches the running task
    unsafe {
        if *WHOSE_RUNNING == 0 {
            //switches from the original main stack to the first task
            *WHOSE_RUNNING = 1;
            context_switch_orig(TASK_STACK_POINTERS[0], &TASK_STACK_POINTERS[0]);
        
        } /*else if *WHOSE_RUNNING == NUM_TASKS
        //switches from the last task to the first
        {
            let index = *WHOSE_RUNNING - 1;
            *WHOSE_RUNNING = 1;
            context_switch(
                &TASK_STACK_POINTERS[index],
                TASK_STACK_POINTERS[0],
                &TASK_STACK_POINTERS[0],
            );
        } else {
            //switches to the next task in the task array
            *WHOSE_RUNNING = *WHOSE_RUNNING + 1;
            context_switch(
                &TASK_STACK_POINTERS[*WHOSE_RUNNING - 2],
                TASK_STACK_POINTERS[*WHOSE_RUNNING - 1],
                &TASK_STACK_POINTERS[*WHOSE_RUNNING - 1],
            );
        }
        */
    }
}

#[naked]
#[no_mangle]
/*
    This function performs the context switch between two tasks, it:
    - saves the current LR and registers r4-11 to the old stack
    - saves the current value of the sp to the right entry in the task pointers array
    - switches to the new stack
    - restores LR and registers r4-11 from the values in the new stack
    - saves the new sp to the right entry of the task pointers array
    - returns with the restored LR value
*/
pub unsafe extern "C" fn context_switch(old_addy: &u32, new_ptr: u32, new_addy: &u32) {
    asm!(
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
/*
    This function performs the initial context switch the original main stack
    and the first task, it:
    - switches to the new stack
    - restores LR and registers r4-11 from the values in the new stack
    - saves the new sp to the right entry of the task pointers array
    - returns with the restored LR value
*/
pub unsafe extern "C" fn context_switch_orig(ptr: u32, prt_addy: &u32) {
    //pointer in r0, pc in r1
    asm!(
        "MSR MSP, r0",
        "POP {{r4-r11}}",
        "POP {{LR}}",
        "STR SP, [r1]",
        "blx LR",
        options(noreturn),
    );
}

#[naked]
#[no_mangle]
/* This is the function that is loaded into the LR value of the initial "saved
registers" portion of each stack. It performs an exception return */
pub unsafe extern "C" fn trampoline() {
    asm!("ldr lr, =0xfffffff9", "bx lr", options(noreturn),);
}

/* This function creates the stacks for all the tasks that need to be run
It takes and array of size NUM_TASKS with all the adresses of the tasks
to be created */
fn initialize_task(func_ptr:u32, stack_size:u32) -> u8 {
    //creates stack pointer for each task and puts them in the TASK_STACK_POINTERS array
        
    //making sure stack size is 4 byte aligned
    if stack_size % 4 != 0 { 
        return 1;
    }

    //Only loading a singular task
    let mut stack_prt = STACK_BASE;

    //makes sure that there is enough RAM to create all the tasks
    if stack_prt > STACK_MAX {
        return 1;
    }
    //gets the stack pointer to the newly created stack and puts in the stack pointer array
    stack_prt = create_stack(stack_prt, func_ptr);

    //initializing fields in task info array
    unsafe {
        TASK_INFO_ARRAY[1].stack_pointer = stack_prt;
        TASK_INFO_ARRAY[1].loaded = true;
        TASK_INFO_ARRAY[1].awake = true;
        TASK_INFO_ARRAY[1].stack_size = stack_size;
    }
    return 0;
}

/* This function creates a task stack given the adress of the task and the
stack pointer */
fn create_stack(sp: u32, func_ptr: u32) -> u32 {
    let mut my_sp = sp;
    let xpcr = 1 << 24; //24th bit is 1
    let dummy_val = 0;
    let all_regs_lr: u32 = trampoline as u32; //initial LR is the trampoline function
    unsafe {
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

//This enables the TIM3 interrupt, which acts as the scheduler
fn start_scheduler() {
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim3) = MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut() {
            //enable the timer
            tim3.cr1.write(|w| w.cen().set_bit());
        }
    });
}

#[no_mangle]
fn delay(delay_10ms: u32) {
    let mut end_loop = false;
    let mut task_id: usize = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut ref mut task_tick = TASK_TICK_COUNTER.borrow(cs).borrow_mut().deref_mut() {
            //zero the task counter
            task_id = TASK_RUNNING.borrow(cs).get() - 1;
            task_tick[task_id] = 0;
        }
    });

    loop {
        free(|cs| {
            // Obtain all Mutex protected resources
            if let &mut ref mut task_tick = TASK_TICK_COUNTER.borrow(cs).borrow_mut().deref_mut() {
                if task_tick[task_id] >= delay_10ms {
                    end_loop = true;
                }
            }
        });
        //if the timer has expired, break from the loop and return
        if end_loop {
            break;
        }
    }
}

fn zero_task_time() {
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut ref mut task_tick = TASK_TICK_COUNTER.borrow(cs).borrow_mut().deref_mut() {
            //zero the task counter
            task_tick[TASK_RUNNING.borrow(cs).get() - 1] = 0;
        }
    });
}

fn get_task_time() -> u32 {
    let mut task_time: u32 = 0;
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut ref mut task_tick = TASK_TICK_COUNTER.borrow(cs).borrow_mut().deref_mut() {
            //return the task time conter value
            task_time = task_tick[TASK_RUNNING.borrow(cs).get() - 1];
        }
    });
    return task_time;
}
#[no_mangle]
fn set_led(color: u32, state: bool) {
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut gpiod) = 
            MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
        
            if color == GREEN {
                if state {
                    gpiod.odr.modify(|_, w| w.odr12().set_bit());
                }
                else {
                    gpiod.odr.modify(|_, w| w.odr12().clear_bit());
                }
            }
            else if color == ORANGE {
                if state {
                    gpiod.odr.modify(|_, w| w.odr13().set_bit());
                }
                else {
                    gpiod.odr.modify(|_, w| w.odr13().clear_bit());
                }
            }
            else if color == RED {
                if state {
                    gpiod.odr.modify(|_, w| w.odr14().set_bit());
                }
                else {
                    gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                }
            }
            else if color == BLUE {
                if state {
                    gpiod.odr.modify(|_, w| w.odr15().set_bit());
                }
                else {
                    gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                }
            }
                       
        }
    });
}



