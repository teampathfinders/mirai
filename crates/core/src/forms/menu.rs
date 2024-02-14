use serde::ser::SerializeStruct;

use crate::forms::Button;

use super::{FormDesc, SubmittableForm};

/// A forms is similar to a modal but it has an arbitrary amount of buttons.
#[derive(Debug)]
pub struct Menu<'a> {
    /// Title of the forms. This is displayed at the top of the window.
    title: &'a str,
    /// Content of the form. This is the text shown above the buttons.
    body: &'a str,
    /// List of buttons that are available.
    buttons: Vec<Button>,
}

impl<'a> Menu<'a> {
    /// Creates a new, empty menu.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the form.
    ///
    /// Default: "Menu"
    pub fn title<I: Into<&'a str>>(mut self, title: I) -> Self {
        // The `Into` trait is used instead of `AsRef` to be able to attach
        // the lifetime to the str.

        self.title = title.into();
        self
    }

    /// Sets the body of the form.
    /// Default: ""
    pub fn body<I: Into<&'a str>>(mut self, body: I) -> Self {
        // The `Into` trait is used instead of `AsRef` to be able to attach
        // the lifetime to the str.

        self.body = body.into();
        self
    }

    /// Adds a button to the menu.
    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }
}

impl Default for Menu<'_> {
    fn default() -> Self {
        Self {
            title: "Menu",
            body: "",
            buttons: Vec::new(),
        }
    }
}

impl SubmittableForm for Menu<'_> {
    fn into_desc(self) -> FormDesc {
        FormDesc::Menu
    }
}

impl serde::Serialize for Menu<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("form", 4)?;
        map.serialize_field("type", "form")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.body)?;
        map.serialize_field("buttons", &self.buttons)?;
        map.end()
    }
}
