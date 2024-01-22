use std::collections::HashMap;

use serde::ser::SerializeStruct;

use crate::forms::Content;

use super::{FormDesc, SubmittableForm};

mod private {
    use crate::forms::{Dropdown, Input, Label, Slider, StepSlider, Toggle};

    use super::Content;

    pub trait Sealed: Into<Content> {}

    impl Sealed for Label {}
    impl Sealed for Input {}
    impl Sealed for Toggle {}
    impl Sealed for Dropdown {}
    impl Sealed for Slider {}
    impl Sealed for StepSlider {}
}

/// Any item that can be used as content in a custom form. All the UI elements apart from buttons (label, input, etc.) support this.
pub trait Submittable: private::Sealed {}

/// A forms with a custom body.
/// Unlike the other forms types, this forms can make use of all the custom UI elements.
#[derive(Debug)]
pub struct Custom<'a> {
    /// Title displayed at the top of the window.
    title: &'a str,
    /// List of custom elements.
    content: HashMap<String, Content>,
}

impl<'a> Default for Custom<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Custom<'a> {
    /// Creates a new form.
    pub fn new() -> Self {
        Self { title: "Form", content: HashMap::new() }
    }

    /// Sets the title of the form.
    ///
    /// Default: "Form".
    pub fn title(mut self, title: impl Into<&'a str>) -> Self {
        self.title = title.into();
        self
    }

    /// Adds an element to the body of the form.
    pub fn with(mut self, key: impl Into<String>, submittable: impl Submittable) -> Self {
        self.content.insert(key.into(), submittable.into());
        self
    }
}

impl<'a> SubmittableForm for Custom<'a> {
    fn into_desc(self) -> FormDesc {
        FormDesc::Custom(self.content)
    }
}

impl serde::Serialize for Custom<'_> {
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
