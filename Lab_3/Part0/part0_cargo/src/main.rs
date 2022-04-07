use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() {
    //declaring the total primes and vector to store the threads
    let mut handles = vec![];
    let primes = Arc::new(Mutex::new(0));

    //collect the command line arguments
    let args: Vec<String> = env::args().collect();
    let n_string = &args[1];
    let thread_no = &args[2];

    //convert command line args from strings to numbers
    let n: u32 = n_string.trim().parse().expect("Please enter a number!");
    let thread_no: u32 = thread_no.trim().parse().expect("Please enter a number!");

    println!("Threads: {}", thread_no);

    //declares sieve array mutex
    let sieve = Arc::new(Mutex::new(vec![true; n.try_into().unwrap()]));

    //starts runtime clock
    let now = Instant::now();
    if n < 3 {
        println!("Primes smaller than {}: {}", n, 0);
        println!("Time taken: {} ns", now.elapsed().as_nanos());
        return;
    }

    //spawning the threads
    for i in 0..thread_no {
        //creating copies of variables so the original ownership of the variables is not moved into the thread
        let primes_clone = Arc::clone(&primes);
        let sieve_clone = Arc::clone(&sieve);
        let new_n = n;
        let new_thread_no = thread_no;
        let start = i;

        //spawning the threads
        let handle = thread::spawn(move || {
            let mut my_num = start + 2; //0 and 1 are not primes
            let mut primes_recorded = 0;
            while my_num < new_n {
                //get acess to sieve array
                let mut my_sieve_clone = sieve_clone.lock().unwrap();
                if my_sieve_clone[my_num as usize] {
                    primes_recorded += 1;
                    let mut scalar = 2;
                    while (my_num * scalar) < new_n {
                        my_sieve_clone[(my_num * scalar) as usize] = false;
                        scalar += 1;
                    }
                }
                //let other threads access sieve array
                drop(my_sieve_clone);
                my_num += new_thread_no;
            }
            //update global primes count before thread dies
            let mut my_primes_clone = primes_clone.lock().unwrap();
            *my_primes_clone += primes_recorded;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let primes_clone = Arc::clone(&primes);
    let primes_final = primes_clone.lock().unwrap();
    println!("Primes smaller than {}: {}", n, *primes_final);
    println!("Time taken: {} ns", now.elapsed().as_nanos());
    return;
}
