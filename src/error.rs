/// Verifies that the given expression evaluates to true,
/// or returns an error
#[macro_export]
macro_rules! vex_assert {
    ($expression: expr, $message: expr) => {
        if ($expression) == false {
            anyhow::bail!("Assertion failed: {} | {}", $expression, $message);
        }
    };

    ($expression: expr) => {
        vex_assert!(
            $expression,
            format!("Assertion failed: {}", stringify!($expression))
        );
    };
}
