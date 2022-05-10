//Flash orange led task

//Color LED color constants (defined in the kernel source code)
//const GREEN: u32 = 1;
const ORANGE: u32 = 2;
//const RED: u32 = 3;
//const BLUE: u32 = 4;
use panic_halt as _;

//methods defined in the kernel source code for access to peripherals, task scheduling control
extern "C" {
    fn set_led(color: u32, state: bool);
    fn sleep(delay_10ms: u32);
}

//based on the agreement with the kernel, this is the never returning method that
//is the "main" function for this task
//Practically, this method flashes the orange led every second
#[no_mangle]
pub extern "C" fn start() {
    let mut led_on = false;
    loop {
        //toggle the led value
        led_on = !led_on;

        unsafe {
            //set the led to the new value
            set_led(ORANGE, !led_on);

            //delay for 1 sec
            sleep(100);
        }
    }
}
