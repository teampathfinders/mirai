use serde::ser::SerializeStruct;

use crate::forms::FormElement;

/// A forms with a custom body.
/// Unlike the other forms types, this forms can make use of all the custom UI elements.
#[derive(Debug)]
pub struct CustomForm<'a> {
    /// Title displayed at the top of the window.
    pub title: &'a str,
    /// List of custom elements.
    pub content: &'a [FormElement<'a>],
}

impl<'a> serde::Serialize for CustomForm<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("custom_form", 3)?;
        map.serialize_field("type", "custom_form")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.content)?;
        map.end()
    }
}
