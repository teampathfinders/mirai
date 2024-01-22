use serde::ser::SerializeStruct;

use crate::forms::FormButton;

use super::{FormDescriptor, SubmittableForm};

/// A forms is similar to a modal but it has an arbitrary amount of buttons.
#[derive(Debug)]
pub struct MenuForm<'a> {
    /// Title of the forms. This is displayed at the top of the window.
    title: &'a str,
    /// Content of the form. This is the text shown above the buttons.
    body: &'a str,
    /// List of buttons that are available.
    buttons: Vec<FormButton>,
}

impl<'a> MenuForm<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the form.
    ///
    /// Default: "Menu"
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Sets the body of the form.
    /// Default: ""
    pub fn body(mut self, body: &'a str) -> Self {
        self.body = body;
        self
    }

    /// Adds a button to the menu.
    pub fn button(mut self, button: FormButton) -> Self {
        self.buttons.push(button);
        self
    }
}

impl Default for MenuForm<'_> {
    fn default() -> Self {
        Self {
            title: "Menu",
            body: "",
            buttons: Vec::new(),
        }
    }
}

impl SubmittableForm for MenuForm<'_> {
    fn into_descriptor(self) -> FormDescriptor {
        FormDescriptor::Menu
    }
}

impl serde::Serialize for MenuForm<'_> {
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
