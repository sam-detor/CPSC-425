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

//Peripheral mutexes
static MUTEX_GPIOD: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOD>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM2: Mutex<RefCell<Option<stm32f4::stm32f411::TIM2>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM3: Mutex<RefCell<Option<stm32f4::stm32f411::TIM3>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_GPIOC: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOC>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_GPIOA: Mutex<RefCell<Option<stm32f4::stm32f411::GPIOA>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_EXTI: Mutex<RefCell<Option<stm32f4::stm32f411::EXTI>>> =
    Mutex::new(RefCell::new(None));

static MUTEX_TIM4: Mutex<RefCell<Option<stm32f4::stm32f411::TIM4>>> =
    Mutex::new(RefCell::new(None));

//Scheduler Constants
const STACK_BASE: u32 = 0x2001F700;
const STACK_SIZE: u32 = 3072; //3KB
const MAX_TASKS: usize = 5; //4 user tasks, 1 idle task

//LED constants
const GREEN: u32 = 1;
const ORANGE: u32 = 2;
const RED: u32 = 3;
const BLUE: u32 = 4;

//Scheduler data structures
struct TaskInfo {
    loaded: bool,
    awake: bool,
    task_tic: u32,
    stack_start: u32,
    sleep_timer: u32,
}

//Scheduler Mutexes
static TASK_INFO_ARRAY: Mutex<RefCell<Option<[TaskInfo; MAX_TASKS]>>> =
    Mutex::new(RefCell::new(Some([
        TaskInfo {
            loaded: true,
            awake: true,
            task_tic: 0,
            stack_start: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_start: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_start: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_start: 0,
            sleep_timer: 0,
        },
        TaskInfo {
            loaded: false,
            awake: true,
            task_tic: 0,
            stack_start: 0,
            sleep_timer: 0,
        },
    ])));

static TASK_RUNNING: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static TASKS_LOADED: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static FREE_BLOCKS: Mutex<RefCell<Option<[u32; MAX_TASKS - 1]>>> =
    Mutex::new(RefCell::new(Some([0, 0, 0, 0])));

//Scheduler mutable statics
static mut TASK_POINTERS_ARRAY: [u32; MAX_TASKS] = [0; 5];

//PC7 debouncing static variables
static DEBOUNCED: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

//Linked tasks mutexes
static LINKED_TASKS: Mutex<RefCell<Option<[u32; MAX_TASKS - 1]>>> =
    Mutex::new(RefCell::new(Some([0, 0, 0, 0])));
static NUM_LINKED_TASKS: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));

//default values for start method
#[no_mangle]
extern "C" fn default_start() {
    loop {}
}

//definitions of the start functions for all the potential tasks
extern "C" {
    fn start0();
    fn start1();
    fn start2();
    fn start3();
}

#[entry]
fn main() -> ! {
    // Getting access to the peripherals
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm32f4_peripherals = stm32f411::Peripherals::take().unwrap();

    // Enabling GPIOA, GPIOC, GPIOD clocks
    let rcc = &stm32f4_peripherals.RCC;
    rcc.ahb1enr.write(|w| {
        w.gpioaen()
            .set_bit()
            .gpiocen()
            .set_bit()
            .gpioden()
            .set_bit()
    });

    //enable syscfg clock
    rcc.apb2enr.write(|w| w.syscfgen().set_bit());

    //Enable tim2, tim3, tim4 clock
    rcc.apb1enr
        .write(|w| w.tim2en().set_bit().tim3en().set_bit().tim4en().set_bit());

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

    // Configure PC7 pin as input, floating
    let gpioc = &stm32f4_peripherals.GPIOC;
    gpioc.moder.write(|w| w.moder7().input());
    gpioc.pupdr.write(|w| w.pupdr7().floating());

    //configure PA0 as an input, floating
    let gpioa = &stm32f4_peripherals.GPIOA;
    gpioa.moder.write(|w| w.moder0().input());
    gpioa.pupdr.write(|w| w.pupdr0().floating());

    //Setting up scheduler
    //get access to tim3
    let tim2 = &stm32f4_peripherals.TIM2;
    let tim3 = &stm32f4_peripherals.TIM3;
    let tim4 = &stm32f4_peripherals.TIM4;

    //set prescalar values
    //to turn an 8mHz clock into 1ms intervals
    tim2.psc.write(|w| w.psc().bits(15999));
    tim3.psc.write(|w| w.psc().bits(15999));
    tim4.psc.write(|w| w.psc().bits(15999));

    //set auto refil values
    tim2.arr.write(|w| w.arr().bits(300));
    tim3.arr.write(|w| w.arr().bits(10));
    tim4.arr.write(|w| w.arr().bits(100));

    //enable interrupts
    tim2.dier.write(|w| w.uie().set_bit());
    tim3.dier.write(|w| w.uie().set_bit());
    tim4.dier.write(|w| w.uie().set_bit());

    //set as one pulse timer
    tim2.cr1.write(|w| w.opm().set_bit().cen().clear_bit());

    //set as a repetitive timer
    tim3.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());
    tim4.cr1.write(|w| w.opm().clear_bit().cen().clear_bit());

    //connect EXTI7 line to PC7 pin
    let syscfg = &stm32f4_peripherals.SYSCFG;
    syscfg.exticr2.write(|w| unsafe { w.exti7().bits(0b0010) });

    //connect EXTI0 to PA0 pin/button
    syscfg.exticr1.write(|w| unsafe { w.exti0().bits(0b0000) });

    //configure both EXTI7 and EXTI0 to be enabled and trigger on falling edge
    let exti = &stm32f4_peripherals.EXTI;
    exti.imr.modify(|_, w| w.mr7().set_bit()); // unmask interrupt
    exti.ftsr.modify(|_, w| w.tr7().set_bit()); // falling edge trigger

    exti.imr.modify(|_, w| w.mr0().set_bit()); // unmask interrupt
    exti.ftsr.modify(|_, w| w.tr0().set_bit()); // falling edge trigger

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
        MUTEX_GPIOA
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.GPIOA));
    });

    //initialize the free blocks and linked tasks arrays
    init_free_blocks();
    get_linked_tasks();

    //Enable EXTI0, EXTI7, TIM2, TIM3, TIM4 Interrupts
    let mut nvic = cortexm_peripherals.NVIC;
    unsafe {
        nvic.set_priority(Interrupt::TIM2, 3);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);

        nvic.set_priority(Interrupt::TIM3, 2);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM3);

        nvic.set_priority(Interrupt::TIM4, 3);
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM4);

        nvic.set_priority(Interrupt::EXTI9_5, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI9_5);

        nvic.set_priority(Interrupt::EXTI0, 1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0);
    }

    cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI9_5);
    cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI0);
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM2);
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM3);
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM4);

    //enabling timer 4
    tim4.cr1.write(|w| w.cen().set_bit());

    //moving the timers into the mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_TIM3
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM3));
        MUTEX_TIM4
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM4));
        MUTEX_TIM2
            .borrow(cs)
            .replace(Some(stm32f4_peripherals.TIM2));
    });

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
        if let (&mut Some(ref mut tim3), &mut Some(ref mut task_info_array)) = (
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
                for i in 1..MAX_TASKS {
                    task_info_array[i].task_tic += 1;
                }

                //check to see if any loaded task needs to be woken up
                for i in 1..MAX_TASKS {
                    if task_info_array[i].loaded && !task_info_array[i].awake {
                        if task_info_array[i].task_tic >= task_info_array[i].sleep_timer {
                            task_info_array[i].awake = true;
                        }
                    }
                }

                //select the next task to be run
                let prev_task = TASK_RUNNING.borrow(cs).get();
                if prev_task == MAX_TASKS - 1 {
                    next_task = 1;
                } else if prev_task == 0 {
                    //if it was the idle task, it's assumed every other task was asleep
                    next_task = *PAUSED_TASK; //scheduler should pick up where it left off in that case
                } else {
                    next_task = prev_task + 1;
                }

                //If "next_task" is not awake or loaded, try to find one that is
                if !task_info_array[next_task].loaded || !task_info_array[next_task].awake {
                    *PAUSED_TASK = next_task;
                    loop {
                        if next_task == MAX_TASKS - 1 {
                            next_task = 1;
                        } else {
                            next_task += 1;
                        }

                        if task_info_array[next_task].loaded && task_info_array[next_task].awake {
                            break;
                        } else if next_task == *PAUSED_TASK {
                            //if you have gone through all of the tasks and none are awake and loaded
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
fn initialize_task(func_ptr: u32, task_id: usize) {
    let mut stack_prt: u32 = 0;
    let mut stack_start: u32 = 0;
    free(|cs| {
        if let (&mut Some(ref mut task_info_array), &mut Some(ref mut free_blocks)) = (
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut(),
            FREE_BLOCKS.borrow(cs).borrow_mut().deref_mut(),
        ) {
            //gets a memory chunck from the FREE_BLOCKS to use as a stack for the task
            for i in 0..(MAX_TASKS - 1) {
                if free_blocks[i] != 0 {
                    stack_prt = free_blocks[i];
                    free_blocks[i] = 0;
                    break;
                }
            }
            //save the original start of the stack so the memory can later be retrieved by the unloading method
            stack_start = stack_prt;

            //gets the stack pointer to the newly created stack and puts in the stack pointer array
            stack_prt = create_stack(stack_prt, func_ptr);
            unsafe {
                TASK_POINTERS_ARRAY[task_id] = stack_prt;
            }

            TASKS_LOADED
                .borrow(cs)
                .set(TASKS_LOADED.borrow(cs).get() + 1); // update the TASKS_LOADED var

            //initializing fields in the task info array
            task_info_array[task_id].awake = true;
            task_info_array[task_id].loaded = true;
            task_info_array[task_id].stack_start = stack_start; //saving the start of the stack
        }
    });
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

#[interrupt]
fn TIM4() {
    //triggered every 0.1s, refreshes the charge on PC7
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

fn TIM2() {
    //Debouncing timer for PC7
    //triggered 0.5 after a click of PC7 registered.
    free(|cs| {
        // Obtain all Mutex protected resources
        if let &mut Some(ref mut tim2) = MUTEX_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.sr.write(|w| w.uif().clear_bit()); //clear pending interrupt bit
            DEBOUNCED.borrow(cs).set(true); //allows for the unloading sequence to be triggered again
        }
    });
}

#[interrupt]
fn EXTI9_5() {
    //triggered when PC7 is touched

    static mut NEXT_UNLOAD: usize = 1;

    cortex_m::interrupt::free(|cs| {
        let refcell = MUTEX_EXTI.borrow(cs).borrow();
        let exti = match refcell.as_ref() {
            None => return,
            Some(v) => v,
        };
        if let &mut Some(ref mut tim2) = MUTEX_TIM2.borrow(cs).borrow_mut().deref_mut() {
            // clear the EXTI line 7 pending bit
            exti.pr.write(|w| w.pr7().set_bit());

            //if another click hasn't been recently registered
            if DEBOUNCED.borrow(cs).get() {
                //if there are tasks loaded
                if TASKS_LOADED.borrow(cs).get() > 0 {
                    unload_task(*NEXT_UNLOAD); //*NEXT_UNLOAD keeps track of the oldest loaded task
                    if *NEXT_UNLOAD == NUM_LINKED_TASKS.borrow(cs).get() {
                        *NEXT_UNLOAD = 1;
                    } else {
                        *NEXT_UNLOAD += 1;
                    }
                }
                DEBOUNCED.borrow(cs).set(false);
                tim2.cr1.write(|w| w.cen().set_bit()); //debouncing timer
            }
        }
    });
}

#[interrupt]
fn EXTI0() {
    //triggered when the user button is pressed

    static mut NEXT_LOAD: usize = 1; //keeps track of the next available spot to load a task
                                     //and what the next task is to load in the linked tasks array
    cortex_m::interrupt::free(|cs| {
        let refcell = MUTEX_EXTI.borrow(cs).borrow();
        let exti = match refcell.as_ref() {
            None => return,
            Some(v) => v,
        };
        if let &mut Some(ref mut linked_tasks) = LINKED_TASKS.borrow(cs).borrow_mut().deref_mut() {
            // clear the EXTI line 7 pending bit
            exti.pr.write(|w| w.pr0().set_bit());

            //getting variables from mutexes
            let tasks_loaded = TASKS_LOADED.borrow(cs).get();
            let num_linked_tasks = NUM_LINKED_TASKS.borrow(cs).get();

            //if there are more tasks to load
            if tasks_loaded < num_linked_tasks {
                initialize_task(linked_tasks[*NEXT_LOAD - 1], *NEXT_LOAD);

                //update NEXT_LOAD
                if *NEXT_LOAD == num_linked_tasks {
                    *NEXT_LOAD = 1;
                } else {
                    *NEXT_LOAD += 1;
                }
                //the init function takes care of updating TASKS_LOADED
            }
        }
    });
}

//fills the free blocks array with addresses 3KB apart, that can be used for stack pointers for tasks
fn init_free_blocks() {
    free(|cs| {
        if let &mut Some(ref mut free_blocks) = FREE_BLOCKS.borrow(cs).borrow_mut().deref_mut() {
            for i in 0..(MAX_TASKS - 1) {
                free_blocks[i] = STACK_BASE - (i as u32 * STACK_SIZE);
            }
        }
    });
}

//takes all the function addresses of the linked tasks and puts them into the LINKED_TASKS array
fn get_linked_tasks() {
    let mut index = 0;
    free(|cs| {
        if let &mut Some(ref mut linked_tasks) = LINKED_TASKS.borrow(cs).borrow_mut().deref_mut() {
            //task 1
            if start0 as u32 != default_start as u32 {
                linked_tasks[index] = start0 as u32;
                index += 1;
            }

            //task 2
            if start1 as u32 != default_start as u32 {
                linked_tasks[index] = start1 as u32;
                index += 1;
            }

            //task 3
            if start2 as u32 != default_start as u32 {
                linked_tasks[index] = start2 as u32;
                index += 1;
            }

            //task 4
            if start3 as u32 != default_start as u32 {
                linked_tasks[index] = start3 as u32;
                index += 1;
            }

            //set the number of linked tasks variable
            NUM_LINKED_TASKS.borrow(cs).set(index);
        }
    });
}

//This function takes a task of "task_id" and sets its loaded field in the task info array to false, returns that task's
//original stack pointer to the FREE_BLOCKS array, and updates the TASKS_LOADED variable
fn unload_task(task_id: usize) {
    free(|cs| {
        if let (&mut Some(ref mut task_info_array), &mut Some(ref mut free_blocks)) = (
            TASK_INFO_ARRAY.borrow(cs).borrow_mut().deref_mut(),
            FREE_BLOCKS.borrow(cs).borrow_mut().deref_mut(),
        ) {
            task_info_array[task_id].loaded = false;

            TASKS_LOADED
                .borrow(cs)
                .set(TASKS_LOADED.borrow(cs).get() - 1);

            //returns the stack pointer to the free_blocks array
            for i in 0..(MAX_TASKS - 1) {
                if free_blocks[i] == 0 {
                    free_blocks[i] = task_info_array[task_id].stack_start;
                    break;
                }
            }
        }
    });
}
