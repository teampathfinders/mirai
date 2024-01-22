use std::collections::HashMap;

use serde::ser::SerializeStruct;

use super::{FormDescriptor, FormElement, FormLabel, FormVariant, SubmittableForm};

/// A modal is a forms that only has a body and two buttons.
/// Unlike [`CustomForm`](crate::forms::CustomForm)
/// [`FormButton`](crate::forms::FormButton)s, these buttons cannot have images next to them.
#[derive(Debug)]
pub struct ModalForm<'a> {
    /// Title displayed at the top of the window.
    title: &'a str,
    /// Text displayed in the modal.
    body: &'a str,
    /// Text body of the first button.
    first: &'a str,
    /// Text body of the second button.
    second: &'a str,
}

impl<'a> ModalForm<'a> {
    /// Creates a new default modal.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the modal.
    ///
    /// Default: "Modal".
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Sets the body of the modal.
    ///
    /// Default: ""
    pub fn body(mut self, body: &'a str) -> Self {
        self.body = body;
        self
    }

    /// Sets the text of the first button of the modal.
    ///
    /// Default: "Confirm".
    pub fn first(mut self, first: &'a str) -> Self {
        self.first = first;
        self
    }

    /// Sets the text of the second button of the modal.
    ///
    /// Default: "Cancel".
    pub fn second(mut self, second: &'a str) -> Self {
        self.second = second;
        self
    }
}

impl Default for ModalForm<'_> {
    fn default() -> Self {
        Self {
            title: "Modal",
            body: "",
            first: "Confirm",
            second: "Cancel",
        }
    }
}

impl SubmittableForm for ModalForm<'_> {
    fn into_descriptor(self) -> FormDescriptor {
        let content = HashMap::from([
            ("first".to_owned(), FormElement::Label(FormLabel { label: String::new() })),
            ("second".to_owned(), FormElement::Label(FormLabel { label: String::new() })),
        ]);

        FormDescriptor { variant: FormVariant::Modal, content }
    }
}

impl<'a> serde::Serialize for ModalForm<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("modal", 5)?;
        map.serialize_field("type", "modal")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.body)?;
        map.serialize_field("button1", self.first)?;
        map.serialize_field("button2", self.second)?;
        map.end()
    }
}
