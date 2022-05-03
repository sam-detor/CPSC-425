//flash green led
const GREEN: u32 = 1;
//const ORANGE: u32 = 2;
//const RED: u32 = 3;
//const BLUE: u32 = 4;
use panic_halt as _;

extern "C" {
    fn set_led(color: u32, state: bool);
    fn sleep(delay_10ms: u32);
}

#[no_mangle]
pub fn start() {
    let mut led_on = false;
    loop {
            unsafe {
                set_led(GREEN, !led_on);
            }

            led_on = !led_on;
        
        unsafe {
        //delay for 1 sec
        sleep(100);
        }
    }

}

#[no_mangle]
pub extern "C" fn task_stack_size_getter() -> u32 {
    return 2048;
}
