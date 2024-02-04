use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder()
        .all_git()
        .emit()
        .expect("Failed to collect build info");
}