#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(type_ascription)]
#![feature(linkage)]

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

//Peripheral mutexes
static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

//Scheduler Constants
const STACK_BASE: u32 = 0x2001F700;
const STACK_SIZE_MAX: u32 = 4096;
const MAX_TASKS: usize = 5; //4 user tasks, 1 idle task

//LED constants
const GREEN: u32 = 1;
const ORANGE: u32 = 2;
const RED: u32 = 3;
const BLUE: u32 = 4;

struct TaskInfo {
    loaded: bool,
    awake: bool,
    task_tic: u32,
    stack_min: u32,
    sleep_timer: u32,
}

//Scheduler Mutexes
static TASK_INFO_ARRAY: Mutex<RefCell<Option<[TaskInfo; MAX_TASKS]>>> =
    Mutex::new(RefCell::new(Some([
        TaskInfo {
            loaded: true,
            awake: true,
            task_tic: 0,
            stack_min: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_min: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_min: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_min: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_min: 0,
            sleep_timer: 0,
        },
    ])));

static TASK_RUNNING: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static TASKS_LOADED: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));

//Scheduler mutable statics
static mut TASK_POINTERS_ARRAY: [u32; MAX_TASKS] = [0; 5];

//default values for stack size and start method
#[no_mangle]
static default_stack_size: u32 = 0;

#[no_mangle]
extern "C" fn default_start() {
    loop {}
}

extern "C" {
    fn start0();
    fn start1();
    fn start2();
    fn start3();

    static task0_stack_size: u32;
    static task1_stack_size: u32;
    static task2_stack_size: u32;
    static task3_stack_size: u32;
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

    //loading tasks
    unsafe {
        //task 1
        if task0_stack_size != 0 {
            initialize_task(start0 as u32, task0_stack_size);
        }

        //task 2
        if task1_stack_size != 0 {
            initialize_task(start1 as u32, task1_stack_size);
        }

        //task 3
        if task2_stack_size != 0 {
            initialize_task(start2 as u32, task2_stack_size);
        }

        //task 4
        if task3_stack_size != 0 {
            initialize_task(start3 as u32, task3_stack_size);
        }
    }

    start_scheduler();

    loop {}
}

#[interrupt]
fn TIM3() {
    //triggeres every 10ms and switches the running task

    static mut WHOSE_RUNNING: usize = 0;
    static mut PAUSED_TASK: usize = 1;
    let mut next_task: usize = 1;

    free(|cs| {
        // Obtain all Mutex protected resources
        if let (
            &mut Some(ref mut tim3),
            &mut Some(ref mut task_info_array),
        ) = (
            MUTEX_TIM3.borrow(cs).borrow_mut().deref_mut(),
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut(),
        ) {
            tim3.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit

            //gets the number of tasks loaded
            let num_tasks = TASKS_LOADED.borrow(cs).get();
            if num_tasks == 0 {
                //if there are no user tasks, run the idle task forever
                next_task = 0;
            } else {
                //add update all the task timers
                for i in 1..=num_tasks {
                    task_info_array[i].task_tic += 1;
                }

                //check to see if any task needs to be woken up
                for i in 1..=num_tasks {
                    if !task_info_array[i].awake {
                        if task_info_array[i].task_tic >= task_info_array[i].sleep_timer {
                            task_info_array[i].awake = true;
                        }
                    }
                }

                //select the next task to be run
                let prev_task = TASK_RUNNING.borrow(cs).get();
                if prev_task == num_tasks {
                    next_task = 1;
                } else if prev_task == 0 {
                    //if it was the idle task, it's assumed every other task was asleep
                    next_task = *PAUSED_TASK; //scheduler should pick up where it left off in that case
                } else {
                    next_task = prev_task + 1;
                }

                //If "next_task" is not awake, try to find one that is
                if !task_info_array[next_task].awake {
                    *PAUSED_TASK = next_task;
                    loop {
                        if next_task == num_tasks {
                            next_task = 1;
                        } else {
                            next_task += 1;
                        }

                        if task_info_array[next_task].awake {
                            break;
                        } else if next_task == *PAUSED_TASK {
                            //if you have gone through all of the tasks and none are awake
                            next_task = 0; //run the idle task
                            break;
                        }
                    }
                }
            }
            //update the TASK_RUNNING variable
            TASK_RUNNING.borrow(cs).set(next_task)
        }
    });

    //switches the running task
    if *WHOSE_RUNNING != next_task {
        //if a new task has to be run, switch the tasks
        let temp_whoose_running = *WHOSE_RUNNING; //have the update *WHOSE_RUNNING before the context switch call, so need a temp
        *WHOSE_RUNNING = next_task; //var to store the value
        unsafe {
            context_switch(
                &TASK_POINTERS_ARRAY[temp_whoose_running],
                TASK_POINTERS_ARRAY[next_task],
                &TASK_POINTERS_ARRAY[next_task],
            );
        }
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
/* This is the function that is loaded into the LR value of the initial "saved
registers" portion of each stack. It performs an exception return */
pub unsafe extern "C" fn trampoline() {
    asm!("ldr lr, =0xfffffff9", "bx lr", options(noreturn),);
}

/* This function fills in all necessary fields in this new task's "TaskInfo" struct (ex.
 marking the task as loaded and awake).
It also creates a stack and puts this task's stack pointer into the TASK_POINTERS_ARRAY. */
fn initialize_task(func_ptr: u32, stack_size: u32) -> u8 {
    let mut task_id: usize = 0;
    let mut stack_prt: u32 = 0;
    let mut stack_start: u32 = 0;
    free(|cs| {
        if let &mut Some(ref mut task_info_array) =
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut()
        {
            //calculates this task's id
            task_id = TASKS_LOADED.borrow(cs).get() + 1;

            if task_id == 1 {
                //if this is the first task loaded, make the stack_start the STACK BASE var
                stack_prt = STACK_BASE;
                stack_start = stack_prt;
            } else {
                //make the stack_min from the previous max the stack start (minue 4 for some space between tasks)
                stack_prt = task_info_array[task_id - 1].stack_min - 4; //a little space between each task
                stack_start = stack_prt;
            }
        }
    });

    //making sure stack_min will be 4 byte aligned and is under the max stack size allowed
    if stack_size % 4 != 0 || stack_size > STACK_SIZE_MAX {
        return 1;
    }

    //gets the stack pointer to the newly created stack and puts in the stack pointer array
    stack_prt = create_stack(stack_prt, func_ptr);
    unsafe {
        TASK_POINTERS_ARRAY[task_id] = stack_prt;
    }

    free(|cs| {
        if let &mut Some(ref mut task_info_array) =
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut()
        {
            TASKS_LOADED
                .borrow(cs)
                .set(TASKS_LOADED.borrow(cs).get() + 1); // update the TASKS_LOADED var
                                                         //initializing fields in the task info array
            task_info_array[task_id].stack_min = stack_start - stack_size;
            task_info_array[task_id].awake = true;
            task_info_array[task_id].loaded = true;
        }
    });

    return 0;
}

#[no_mangle]
/* This function creates a task stack given the address of the task and the
stack pointer */
pub fn create_stack(sp: u32, func_ptr: u32) -> u32 {
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
//this function can be called by user tasks. It sets the current task to not be
//run for "delay_10ms" system ticks.
fn sleep(delay_10ms: u32) {
    let mut task_id: usize = 0;
    let mut end_loop: bool = false;
    free(|cs| {
        if let &mut Some(ref mut task_info_array) =
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut()
        {
            //get the current task_id
            task_id = TASK_RUNNING.borrow(cs).get();

            //zero the current task_tic timer, set the task as asleep, set the sleep timer value
            task_info_array[task_id].task_tic = 0;
            task_info_array[task_id].sleep_timer = delay_10ms;
            task_info_array[task_id].awake = false;
        }
    });
    //this loop prevents the task from exiting this function until the scheduler sets it
    //back to awake.
    loop {
        free(|cs| {
            if let &mut Some(ref mut task_info_array) =
                TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut()
            {
                if task_info_array[task_id].awake {
                    end_loop = true;
                }
            }
        });

        if end_loop {
            break;
        }
    }
}

#[allow(dead_code)]
//This function zeros the task_tic counter for the running task
fn zero_task_time() {
    free(|cs| {
        if let &mut Some(ref mut task_info_array) =
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut()
        {
            let task_id = TASK_RUNNING.borrow(cs).get();
            task_info_array[task_id].task_tic = 0;
        }
    });
}

#[allow(dead_code)]
//This function returns the task_tic counter value for the running task
fn get_task_time() -> u32 {
    let mut task_time: u32 = 0;
    free(|cs| {
        if let &mut Some(ref mut task_info_array) =
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut()
        {
            let task_id = TASK_RUNNING.borrow(cs).get();
            task_time = task_info_array[task_id].task_tic;
        }
    });
    return task_time;
}
#[no_mangle]
//this function can be called by task functions to either set or unset a given LED
//based on the color codes defined in this file
fn set_led(color: u32, state: bool) {
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut gpiod) = MUTEX_GPIOD.borrow(cs).borrow_mut().deref_mut() {
            if color == GREEN {
                if state {
                    gpiod.odr.modify(|_, w| w.odr12().set_bit());
                } else {
                    gpiod.odr.modify(|_, w| w.odr12().clear_bit());
                }
            } else if color == ORANGE {
                if state {
                    gpiod.odr.modify(|_, w| w.odr13().set_bit());
                } else {
                    gpiod.odr.modify(|_, w| w.odr13().clear_bit());
                }
            } else if color == RED {
                if state {
                    gpiod.odr.modify(|_, w| w.odr14().set_bit());
                } else {
                    gpiod.odr.modify(|_, w| w.odr14().clear_bit());
                }
            } else if color == BLUE {
                if state {
                    gpiod.odr.modify(|_, w| w.odr15().set_bit());
                } else {
                    gpiod.odr.modify(|_, w| w.odr15().clear_bit());
                }
            }
        }
    });
}
