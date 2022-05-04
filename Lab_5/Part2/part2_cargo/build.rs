fn main() {
    //search for libs in task_bins dir
    println!("cargo:rustc-link-search=native=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins");

    //link all 4 libs
    println!("cargo:rustc-link-lib=static=flash_blue");
    println!("cargo:rustc-link-lib=static=flash_red");
    println!("cargo:rustc-link-lib=static=flash_green");
    println!("cargo:rustc-link-lib=static=flash_orange");
    
    //rerun build script if any of the 4 libs change
    println!("cargo:rerun-if-changed=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_blue.a");
    println!("cargo:rerun-if-changed=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_green.a");
    println!("cargo:rerun-if-changed=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_red.a");
    println!("cargo:rerun-if-changed=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part2/task_bins/libflash_orange.a");
    
    
}
