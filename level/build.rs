#[cfg(not(skip_leveldb))]
fn main() {
    println!("cargo:rerun-if-env-changed=skip-leveldb");
    println!("cargo:rerun-if-changed=leveldb/leveldb.cpp");

    let dst = cmake::Config::new("leveldb").profile("Release").build();

    println!("cargo:rustc-link-search=native={}/build/out", dst.display());
    println!("cargo:rustc-link-lib=static=leveldb-wrapper");
    println!("cargo:rustc-link-lib=static=leveldb-mcpe");

    #[cfg(unix)]
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg(skip_leveldb)]
fn main() {
    // Speed up docs build by not building LevelDB
}
