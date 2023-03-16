// Load the current Git commit hash.
fn main() {
    let mut config = vergen::Config::default();
    *config.git_mut().sha_kind_mut() = vergen::ShaKind::Short;

    vergen::vergen(config).unwrap();
}
