use serde::ser::SerializeStruct;

use super::Form;

/// A modal is a forms that only has a body and two buttons.
/// Unlike [`CustomForm`](crate::forms::CustomForm)
/// [`FormButton`](crate::forms::FormButton)s, these buttons cannot have images next to them.
#[derive(Debug)]
pub struct Modal<'a> {
    /// Title displayed at the top of the window.
    pub title: &'a str,
    /// Text displayed in the modal.
    pub content: &'a str,
    /// Text body of the first button.
    pub button1: &'a str,
    /// Text body of the second button.
    pub button2: &'a str,
}

impl Form for Modal<'_> {}

impl<'a> serde::Serialize for Modal<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("modal", 5)?;
        map.serialize_field("type", "modal")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.content)?;
        map.serialize_field("button1", self.button1)?;
        map.serialize_field("button2", self.button2)?;
        map.end()
    }
}
