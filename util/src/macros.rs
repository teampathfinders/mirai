/// Shorthand for `mod module; pub use module::*;`.
#[macro_export]
macro_rules! glob_export {
    ($module: ident) => {
        mod $module;
        pub use $module::*;
    };
}
