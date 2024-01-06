use std::sync::atomic::Ordering;

use pyro_macros::{atomic_enum};

#[test]
fn test_implicit() {
    #[atomic_enum]
    #[repr(usize)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum ImplicitEnum {
        First = 5,
        Second,
        Third
    }

    let v = ImplicitEnum::First;
    let av: AtomicImplicitEnum = v.into();

    assert_eq!(av.load(Ordering::Relaxed), ImplicitEnum::First);
    
    av.store(ImplicitEnum::Second, Ordering::Relaxed);
    assert_eq!(av.load(Ordering::Relaxed), ImplicitEnum::Second);
}