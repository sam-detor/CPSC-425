use std::time::Instant;  
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
fn main() {

    let mut handles = vec![];
    let primes =Arc::new(Mutex::new(0));

    let args: Vec<String> = env::args().collect();
    let n = &args[1];
    let thread_no = &args[2];

    println!("Threads: {}", thread_no);

    let n: u32 = n.trim().parse().expect("Please enter a number!");
    let thread_no: u32 = thread_no.trim().parse().expect("Please enter a number!");
    
    let now = Instant::now();
    if n < 3 {
        println!("Primes smaller than {}: {}", n, 0);
        println!("Time taken: {} ns", now.elapsed().as_nanos());
        return;
    }

    for i in 0..thread_no {
        let primes_clone = Arc::clone(&primes);
        let new_n = n;
        let new_thread_no = thread_no;
        let start = i;
        let handle = thread::spawn(move || {
        let mut my_num = start + 2;
        let mut primes_recorded = 0;
        while my_num < new_n {
            let mut k = 1;
            if my_num >= 4 {
                let mut j = 2;
                while j * j <= my_num {
                    if my_num % j == 0 {
                        k = 0;
                    }
                    j = j + 1;
                }
                
            }
            primes_recorded += k;

            my_num += new_thread_no;
        }
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



/*
fn main() {

    let mut num_primes = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    let args: Vec<String> = env::args().collect();
    let n = &args[1];
    let thread_no = &args[2];

    println!("Threads: {}", thread_no);

    let n: u32 = n.trim().parse().expect("Please enter a number!");
    let thread_no: u32 = thread_no.trim().parse().expect("Please enter a number!");
    
    let now = Instant::now();
    if n < 3 {
        println!("Primes smaller than {}: {}", n, 0);
        println!("Time taken: {} ns", now.elapsed().as_nanos());
        return;
    }

    let mut currentNumber = Arc::new(Mutex::new(2));

    thread_no = thread_no - 1;
    
    for _ in 0..thread_no {
        let currentNumber_clone =  Arc::clone(&currentNumber);
        let num_primes_clone =  Arc::clone(&num_primes);
        let handle = thread::spawn(move || {
        let mut my_c_num = currentNumber_clone.lock().unwrap();
        let mut my_num = *my_c_num;
        *my_c_num += 1;
        drop(my_c_num);
        while my_num < n {
            let mut k = 1;
            if my_num >= 4 {
                let mut j = 2;
                let mut k = 1;
                while j * j <= my_num {
                    if my_num % j == 0 {
                        k = 0;
                    }
                    j = j + 1;
                }
                
            }

            let mut my_num_primes = num_primes_clone.lock().unwrap();
            *my_num_primes += k;
            drop(my_num_primes);

            let mut my_c_num = currentNumber_clone.lock().unwrap();
            my_num = *my_c_num;
            *my_c_num += 1;
            drop(my_c_num);
        }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
/*
    while currentNumber < n {
        //i must only be accessed by 1 thread at a time
        if currentNumber < 4 {
            num_primes = num_primes + 1;
        }
        else {
            let mut j = 2;
            let mut k = 1;
            while j * j <= i {
                if i % j == 0 {
                    k = 0;
                }
                j = j + 1;
            }
            num_primes = num_primes + k;
        }

        currentNumber = currentNumber + 1
    }
*/
    let num_primes_final = num_primes_clone.lock().unwrap();
    println!("Primes smaller than {}: {}", n, *num_primes_final);
    println!("Time taken: {} ns", now.elapsed().as_nanos());
    return;
}
*/