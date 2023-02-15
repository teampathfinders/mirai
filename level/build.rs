fn main() {
    let dst = cmake::build("leveldb");

    #[cfg(debug_assertions)]
    {
        println!(
            "cargo:rustc-link-search=native={}/build/Debug",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/vendor/Debug",
            dst.display()
        );
    }

    #[cfg(not(debug_assertions))]
    {
        println!(
            "cargo:rustc-link-search=native={}/build/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/vendor/Release",
            dst.display()
        );
    }

    println!("cargo:rustc-link-lib=static=leveldb-wrapper");
    println!("cargo:rustc-link-lib=static=leveldb-mcpe");
}
