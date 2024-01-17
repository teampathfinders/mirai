use std::collections::HashMap;

use serde::ser::SerializeStruct;

use crate::forms::FormElement;

use super::{Form, FormDescriptor};

pub trait Submittable: Into<FormElement> {}

/// A forms with a custom body.
/// Unlike the other forms types, this forms can make use of all the custom UI elements.
#[derive(Debug)]
pub struct CustomForm<'a> {
    /// Title displayed at the top of the window.
    pub(super) title: &'a str,
    /// List of custom elements.
    pub(super) content: HashMap<String, FormElement>,
}

impl<'a> CustomForm<'a> {
    pub fn new() -> Self {
        Self {
            title: "Form",
            content: HashMap::new()
        }
    }

    pub fn title(mut self, title: impl Into<&'a str>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with(mut self, key: impl Into<String>, submittable: impl Submittable) -> Self {
        self.content.insert(key.into(), submittable.into());
        self
    }
}

impl<'a> Form for CustomForm<'a> {
    fn into_descriptor(self) -> FormDescriptor {
        self.content
    }
}

impl serde::Serialize for CustomForm<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("custom_form", 3)?;
        map.serialize_field("type", "custom_form")?;
        map.serialize_field("title", self.title)?;

        let content = self.content.values().collect::<Vec<_>>();
        map.serialize_field("content", &content)?;

        map.end()
    }
}
