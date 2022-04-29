
fn main() {
    println!("cargo:rustc-link-search=native=/Users/samdetor/CPSC-425/CPSC-425/Lab_5/Part1/flash_blue/target/thumbv7em-none-eabi/debug");
    println!("cargo:rustc-link-lib=static=flash_blue");
}
