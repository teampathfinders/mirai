use serde::ser::SerializeStruct;

use super::{FormDesc, SubmittableForm};

/// A modal is a forms that only has a body and two buttons.
/// Unlike [`CustomForm`](crate::forms::CustomForm)'s buttons, these buttons cannot have images next to them.
#[derive(Debug)]
pub struct Modal<'a> {
    /// Title displayed at the top of the window.
    title: &'a str,
    /// Text displayed in the modal.
    body: &'a str,
    /// Text body of the first button.
    confirm: &'a str,
    /// Text body of the second button.
    cancel: &'a str,
}

impl<'a> Modal<'a> {
    /// Creates a new default modal.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the modal.
    ///
    /// Default: "Modal".
    pub fn title<I: Into<&'a str>>(mut self, title: I) -> Self {
        // The `Into` trait is used instead of `AsRef` to be able to attach
        // the lifetime to the str.

        self.title = title.into();
        self
    }

    /// Sets the body of the modal.
    ///
    /// Default: ""
    pub fn body<I: Into<&'a str>>(mut self, body: I) -> Self {
        // The `Into` trait is used instead of `AsRef` to be able to attach
        // the lifetime to the str.

        self.body = body.into();
        self
    }

    /// Sets the text of the confirm button of the modal.
    ///
    /// Default: "Confirm".
    pub fn confirm<I: Into<&'a str>>(mut self, confirm: I) -> Self {
        // The `Into` trait is used instead of `AsRef` to be able to attach
        // the lifetime to the str.

        self.confirm = confirm.into();
        self
    }

    /// Sets the text of the cancel button of the modal.
    ///
    /// Default: "Cancel".
    pub fn cancel<I: Into<&'a str>>(mut self, cancel: I) -> Self {
        // The `Into` trait is used instead of `AsRef` to be able to attach
        // the lifetime to the str.

        self.cancel = cancel.into();
        self
    }
}

impl Default for Modal<'_> {
    fn default() -> Self {
        Self {
            title: "Modal",
            body: "",
            confirm: "Confirm",
            cancel: "Cancel",
        }
    }
}

impl SubmittableForm for Modal<'_> {
    fn into_desc(self) -> FormDesc {
        FormDesc::Modal
    }
}

impl<'a> serde::Serialize for Modal<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("modal", 5)?;
        map.serialize_field("type", "modal")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.body)?;
        map.serialize_field("button1", self.confirm)?;
        map.serialize_field("button2", self.cancel)?;
        map.end()
    }
}
