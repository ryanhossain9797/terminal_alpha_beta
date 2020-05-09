fn main() {
    let path = "./golibs";
    let lib = "people";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}
