use std::process::Command;
use std::io::{self, Write};
fn main () {
    let output = Command::new("ls")
                    .current_dir("/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins")
                    .output()
                    .expect("failed to execute process");
    
    println!("status: {}", output.status);
    let filenames: &Vec<u8> = &output.stdout;
    println!("{}",filenames);
}