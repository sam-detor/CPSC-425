
fn main() {
    println!("cargo:rustc-link-search=native=../task-bin");
    println!("cargo:rustc-link-lib=static=flash_blue");
}
