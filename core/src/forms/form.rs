use std::collections::HashMap;

use super::FormElement;

mod private {
    use crate::forms::{CustomForm, MenuForm, ModalForm};

    pub trait Sealed: serde::Serialize {}

    impl Sealed for ModalForm<'_> {}
    impl Sealed for MenuForm<'_> {}
    impl Sealed for CustomForm<'_> {}
}

#[derive(Debug)]
#[doc(hidden)]
pub enum FormDescriptor {
    Custom(HashMap<String, FormElement>),
    Modal,
    Menu
}

pub trait SubmittableForm: private::Sealed {
    #[doc(hidden)]
    fn into_descriptor(self) -> FormDescriptor;
}
