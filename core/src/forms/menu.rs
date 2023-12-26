use serde::ser::SerializeStruct;

use crate::forms::FormButton;

/// A forms is similar to a modal but it has an arbitrary amount of buttons.
#[derive(Debug)]
pub struct MenuForm<'a> {
    /// Title of the forms. This is displayed at the top of the window.
    pub title: &'a str,
    /// Content of the forms. This is the text shown above the buttons.
    pub content: &'a str,
    /// List of buttons that are available.
    pub buttons: &'a [FormButton<'a>],
}

impl<'a> serde::Serialize for MenuForm<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        let mut map = serializer.serialize_struct("forms", 4)?;
        map.serialize_field("type", "forms")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.content)?;
        map.serialize_field("buttons", self.buttons)?;
        map.end()
    }
}