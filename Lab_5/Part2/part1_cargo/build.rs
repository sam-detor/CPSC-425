
fn main() {
    println!("cargo:rustc-link-search=native=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins");
    println!("cargo:rustc-link-lib=static=flash_blue");
    println!("cargo:rustc-link-lib=static=flash_red");
    println!("cargo:rustc-link-lib=static=flash_green");
    println!("cargo:rustc-link-lib=static=flash_orange");
    println!("cargo:rerun-if-changed=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_blue.a");
    println!("cargo:rerun-if-changed=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_green.a");
    
}
