use std::collections::HashMap;

use super::Content;

mod private {
    use crate::forms::{Custom, Menu, Modal};

    pub trait Sealed: serde::Serialize {}

    impl Sealed for Modal<'_> {}
    impl Sealed for Menu<'_> {}
    impl Sealed for Custom<'_> {}
}

#[derive(Debug)]
#[doc(hidden)]
pub enum FormDesc {
    Custom(HashMap<String, Content>),
    Modal,
    Menu,
}

/// A form that can be submitted to the client.
pub trait SubmittableForm: private::Sealed {
    #[doc(hidden)]
    fn into_desc(self) -> FormDesc;
}
