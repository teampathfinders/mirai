fn main() {
    let dst = cmake::Config::new("leveldb")
        .profile("Release")
        .uses_cxx11()
        .build();

    println!(
        "cargo:rustc-link-search=native={}/build/Release",
        dst.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/build/vendor/Release",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=leveldb-wrapper");
    println!("cargo:rustc-link-lib=static=leveldb-mcpe");
}
