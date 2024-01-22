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
pub enum FormVariant {
    Modal,
    Menu,
    Custom,
}

#[derive(Debug)]
pub struct FormDescriptor {
    pub(super) variant: FormVariant,
    pub(super) content: HashMap<String, FormElement>,
}

pub trait SubmittableForm: private::Sealed {
    #[doc(hidden)]
    fn into_descriptor(self) -> FormDescriptor;
}
