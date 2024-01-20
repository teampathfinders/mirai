use std::collections::HashMap;

use super::FormElement;

pub trait Form: serde::Serialize {
    #[doc(hidden)]
    fn into_descriptor(self) -> FormDescriptor;
}

pub type FormDescriptor = HashMap<String, FormElement>;
