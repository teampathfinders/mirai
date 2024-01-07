use inferno_macros::atomic_enum;

#[atomic_enum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Hello World!
enum Counter {
    One, Two, Three
}

#[test]
fn test() {

}