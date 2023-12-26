use serde::ser::SerializeStruct;

/// A plain piece of text.
#[derive(Debug)]
pub struct FormLabel<'a> {
    /// Text to display.
    pub(crate) label: &'a str,
}

impl<'a> serde::Serialize for FormLabel<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("label", 2)?;
        map.serialize_field("type", "label")?;
        map.serialize_field("text", self.label)?;
        map.end()
    }
}

/// A text input field.
#[derive(Debug)]
pub struct FormInput<'a> {
    /// Label to display above the field.
    pub label: &'a str,
    /// Placeholder to display inside the field when it is empty.
    pub placeholder: &'a str,
    /// Initial state of the field.
    pub initial: &'a str,
}

impl<'a> serde::Serialize for FormInput<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("input", 4)?;
        map.serialize_field("type", "input")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("placeholder", self.placeholder)?;
        map.serialize_field("default", self.initial)?;
        map.end()
    }
}

/// A simple boolean toggle that switches between true and false.
#[derive(Debug)]
pub struct FormToggle<'a> {
    /// Label to display next to the toggle.
    pub(crate) label: &'a str,
    /// Initial state of the toggle.
    pub(crate) initial: bool,
}

impl<'a> serde::Serialize for FormToggle<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("toggle", 3)?;
        map.serialize_field("type", "toggle")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", &self.initial)?;
        map.end()
    }
}

/// A slider that picks numerical values.
#[derive(Debug)]
pub struct FormSlider<'a> {
    /// Label to display above the slider.
    pub(crate) label: &'a str,
    /// Minimum value of the slider.
    pub(crate) min: f64,
    /// Maximum value of the slider.
    pub(crate) max: f64,
    /// Minimum step of the slider.
    pub(crate) step: f64,
    /// Initial state of the slider.
    pub(crate) initial: f64,
}

impl<'a> serde::Serialize for FormSlider<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("slider", 6)?;
        map.serialize_field("type", "slider")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("min", &self.min)?;
        map.serialize_field("max", &self.max)?;
        map.serialize_field("step", &self.step)?;
        map.serialize_field("default", &self.initial)?;
        map.end()
    }
}

/// A dropdown list of selectable options.
#[derive(Debug)]
pub struct FormDropdown<'a> {
    /// Label to display above the menu.
    pub(crate) label: &'a str,
    /// List of options that can be selected.
    /// The dropdown is of type radio and users can therefore only select a single option.
    pub(crate) options: &'a [&'a str],
    /// Initial state of the dropdown.
    pub(crate) initial: i32,
}

impl<'a> serde::Serialize for FormDropdown<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("dropdown", 4)?;
        map.serialize_field("type", "dropdown")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", &self.initial)?;
        map.serialize_field("options", self.options)?;
        map.end()
    }
}

/// Similar to a dropdown, but in slider form.
#[derive(Debug)]
pub struct FormStepSlider<'a> {
    /// Label to display above the slider.
    pub(crate) label: &'a str,
    /// A list of available options.
    /// The user can pick between these options using the slider.
    pub(crate) steps: &'a [&'a str],
    /// Initial state of the step slider.
    pub(crate) initial: i32,
}

impl<'a> serde::Serialize for FormStepSlider<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("step_slider", 4)?;
        map.serialize_field("type", "step_slider")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", &self.initial)?;
        map.serialize_field("steps", self.steps)?;
        map.end()
    }
}

/// An image displayed next to a button.
#[derive(Debug, Copy, Clone)]
pub enum FormButtonImage<'a> {
    /// A URL pointing to an online image.
    Url(&'a str),
    /// A path pointing to an image in an applied resource pack.
    Path(&'a str)
}

/// A simple button with optional image.
#[derive(Debug)]
pub struct FormButton<'a> {
    /// Text displayed on the button.
    pub(crate) label: &'a str,
    /// An optional image shown to the left of the button.
    /// This button can either be a local file from a resource pack or a URL.
    pub(crate) image: Option<FormButtonImage<'a>>,
}

impl<'a> serde::Serialize for FormButton<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Struct with custom serializer to serialize image data.
        struct ImageData<'b> {
            pub img_type: &'b str,
            pub data: &'b str,
        }

        impl<'b> serde::Serialize for ImageData<'b> {
            fn serialize<S1>(&self, serializer: S1) -> Result<S1::Ok, S1::Error>
            where
                S1: serde::Serializer,
            {
                let mut map = serializer.serialize_struct("image", 2)?;
                map.serialize_field("type", self.img_type)?;
                map.serialize_field("data", self.data)?;
                map.end()
            }
        }

        let mut map = serializer.serialize_struct("button", 1)?;
        if let Some(image) = self.image {
            let (img_type, data) = match image {
                FormButtonImage::Path(p) => ("path", p),
                FormButtonImage::Url(u) => ("url", u)
            };

            let data = ImageData { img_type, data };
            map.serialize_field("image", &data)?;
        }
        map.serialize_field("text", self.label)?;
        map.end()
    }
}

/// Abstraction over a form element.
#[derive(Debug)]
pub enum FormElement<'a> {
    /// See [`FormLabel`].
    Label(FormLabel<'a>),
    /// See [`FormInput`].
    Input(FormInput<'a>),
    /// See [`FormToggle`].
    Toggle(FormToggle<'a>),
    /// See [`FormDropdown`].
    Dropdown(FormDropdown<'a>),
    /// See [`FormSlider`].
    Slider(FormSlider<'a>),
    /// See [`FormStepSlider`].
    StepSlider(FormStepSlider<'a>),
    /// See [`FormButton`].
    Button(FormButton<'a>),
}

impl<'a> serde::Serialize for FormElement<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Button(b) => b.serialize(serializer),
            Self::Dropdown(d) => d.serialize(serializer),
            Self::Input(i) => i.serialize(serializer),
            Self::Label(l) => l.serialize(serializer),
            Self::Slider(s) => s.serialize(serializer),
            Self::StepSlider(s) => s.serialize(serializer),
            Self::Toggle(t) => t.serialize(serializer),
        }
    }
}

/// A modal is a form that only has a body and two buttons.
/// Unlike [`CustomForm`] [`FormButton`]s, these buttons cannot have images next to them.
#[derive(Debug)]
pub struct Modal<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub button1: &'a str,
    pub button2: &'a str,
}

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

/// A form is similar to a modal but it has an arbitrary amount of buttons.
/// Unlike [`CustomForm`] [`FormButton`]s, these buttons cannot have images next to them.
#[derive(Debug)]
pub struct Form<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub buttons: &'a [FormButton<'a>],
}

impl<'a> serde::Serialize for Form<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        let mut map = serializer.serialize_struct("form", 4)?;
        map.serialize_field("type", "form")?;
        map.serialize_field("title", self.title)?;
        map.serialize_field("content", self.content)?;
        map.serialize_field("buttons", self.buttons)?;
        map.end()
    }
}

/// A form with a custom body.
/// Unlike the other form types, this form can make use of all the custom UI elements.
#[derive(Debug)]
pub struct CustomForm<'a> {
    pub title: &'a str,
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