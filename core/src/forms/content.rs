use serde::ser::SerializeStruct;

use super::Submittable;

/// A plain piece of text.
#[derive(Debug, Default)]
pub struct Label {
    /// Text to display.
    pub(super) label: String,
}

impl Label {
    /// Creates a new empty label.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the body of this label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl Submittable for Label {}

impl From<Label> for Content {
    fn from(v: Label) -> Self {
        Content::Label(v)
    }
}

impl serde::Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("label", 2)?;
        map.serialize_field("type", "label")?;
        map.serialize_field("text", &self.label)?;
        map.end()
    }
}

/// A text input field.
#[derive(Debug, Default)]
pub struct Input {
    /// Label to display above the field.
    pub(super) label: String,
    /// Placeholder to display inside the field when it is empty.
    pub(super) placeholder: String,
    /// Initial state of the field.
    pub(super) default: String,
}

impl Input {
    /// Creates a new input.
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the placeholder.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the default state.
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = default.into();
        self
    }
}

impl Submittable for Input {}

impl From<Input> for Content {
    fn from(v: Input) -> Self {
        Content::Input(v)
    }
}

impl serde::Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("input", 4)?;
        map.serialize_field("type", "input")?;
        map.serialize_field("text", &self.label)?;
        map.serialize_field("placeholder", &self.placeholder)?;
        map.serialize_field("default", &self.default)?;
        map.end()
    }
}

/// A simple boolean toggle that switches between true and false.
#[derive(Debug, Default)]
pub struct Toggle {
    /// Label to display next to the toggle.
    pub(super) label: String,
    /// Initial state of the toggle.
    pub(super) default: bool,
}

impl Toggle {
    /// Creates a new toggle.
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the default state.
    pub fn default(mut self, default: impl Into<bool>) -> Self {
        self.default = default.into();
        self
    }
}

impl Submittable for Toggle {}

impl From<Toggle> for Content {
    fn from(v: Toggle) -> Self {
        Content::Toggle(v)
    }
}

impl serde::Serialize for Toggle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("toggle", 3)?;
        map.serialize_field("type", "toggle")?;
        map.serialize_field("text", &self.label)?;
        map.serialize_field("default", &self.default)?;
        map.end()
    }
}

/// A slider that picks numerical values.
#[derive(Debug, Default)]
pub struct Slider {
    /// Label to display above the slider.
    pub(super) label: String,
    /// Minimum value of the slider.
    pub(super) min: f64,
    /// Maximum value of the slider.
    pub(super) max: f64,
    /// Minimum step of the slider.
    pub(super) step: f64,
    /// Initial state of the slider.
    pub(super) default: f64,
}

impl Slider {
    /// Creates a new slider.
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the minimum value.
    pub fn min(mut self, min: impl Into<f64>) -> Self {
        self.min = min.into();
        self
    }

    /// Sets the maximum value.
    pub fn max(mut self, max: impl Into<f64>) -> Self {
        self.max = max.into();
        self
    }

    /// Sets the step between values.
    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the default state.
    pub fn default(mut self, default: impl Into<f64>) -> Self {
        self.default = default.into();
        self
    }
}

impl Submittable for Slider {}

impl From<Slider> for Content {
    fn from(v: Slider) -> Self {
        Content::Slider(v)
    }
}

impl serde::Serialize for Slider {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("slider", 6)?;
        map.serialize_field("type", "slider")?;
        map.serialize_field("text", &self.label)?;
        map.serialize_field("min", &self.min)?;
        map.serialize_field("max", &self.max)?;
        map.serialize_field("step", &self.step)?;
        map.serialize_field("default", &self.default)?;
        map.end()
    }
}

/// A dropdown list of selectable options.
#[derive(Debug, Default)]
pub struct Dropdown {
    /// Label to display above the menu.
    pub(super) label: String,
    /// List of options that can be selected.
    /// The dropdown is of type radio and users can therefore only select a single option.
    pub(super) options: Vec<String>,
    /// Initial state of the dropdown.
    pub(super) default: u32,
}

impl Dropdown {
    /// Creates a new dropdown.
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Adds an option.
    pub fn option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Sets the default state.
    pub fn default(mut self, default: impl Into<u32>) -> Self {
        self.default = default.into();
        self
    }
}

impl Submittable for Dropdown {}

impl From<Dropdown> for Content {
    fn from(v: Dropdown) -> Self {
        Self::Dropdown(v)
    }
}

impl serde::Serialize for Dropdown {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("dropdown", 4)?;
        map.serialize_field("type", "dropdown")?;
        map.serialize_field("text", &self.label)?;
        map.serialize_field("default", &self.default)?;
        map.serialize_field("options", &self.options)?;
        map.end()
    }
}

/// Similar to a dropdown, but in slider forms.
#[derive(Debug, Default)]
pub struct StepSlider {
    /// Label to display above the slider.
    pub(super) label: String,
    /// A list of available options.
    /// The user can pick between these options using the slider.
    pub(super) steps: Vec<String>,
    /// Initial state of the step slider.
    pub(super) default: u32,
}

impl StepSlider {
    /// Creates a new step slider.
    pub fn new() -> Self {
        <Self as Default>::default()
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Adds a step.
    pub fn option(mut self, step: impl Into<String>) -> Self {
        self.steps.push(step.into());
        self
    }

    /// Sets the default state
    pub fn default(mut self, default: impl Into<u32>) -> Self {
        self.default = default.into();
        self
    }
}

impl Submittable for StepSlider {}

impl From<StepSlider> for Content {
    fn from(v: StepSlider) -> Self {
        Content::StepSlider(v)
    }
}

impl serde::Serialize for StepSlider {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("step_slider", 4)?;
        map.serialize_field("type", "step_slider")?;
        map.serialize_field("text", &self.label)?;
        map.serialize_field("default", &self.default)?;
        map.serialize_field("steps", &self.steps)?;
        map.end()
    }
}

/// An image displayed next to a button.
#[derive(Debug, Clone)]
pub(crate) enum ButtonImage {
    /// A URL pointing to an online image.
    Url(String),
    /// A path pointing to an image in an applied resource pack.
    Path(String),
}

/// A simple button with optional image.
#[derive(Debug)]
pub struct Button {
    /// Text displayed on the button.
    pub(crate) body: String,
    /// An optional image shown to the left of the button.
    /// This button can either be a local file from a resource pack or a URL.
    pub(crate) image: Option<ButtonImage>,
}

impl Button {
    /// Creates a new button.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the content of the button.
    ///
    /// Default: "Button".
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Sets the image URL. This can point to an online resource.
    ///
    /// Default: None
    pub fn image_url(mut self, url: impl Into<String>) -> Self {
        self.image = Some(ButtonImage::Url(url.into()));
        self
    }

    /// Sets the image path. This should point to an image from a resource pack.
    ///
    /// Default: None
    pub fn image_path(mut self, path: impl Into<String>) -> Self {
        self.image = Some(ButtonImage::Path(path.into()));
        self
    }
}

impl Default for Button {
    fn default() -> Self {
        Self { body: "Button".to_owned(), image: None }
    }
}

impl serde::Serialize for FormButton {
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
                map.serialize_field("type", &self.img_type)?;
                map.serialize_field("data", &self.data)?;
                map.end()
            }
        }

        let mut map = serializer.serialize_struct("button", 1)?;
        if let Some(image) = &self.image {
            let (img_type, data) = match image {
                ButtonImage::Path(p) => ("path", p),
                ButtonImage::Url(u) => ("url", u),
            };

            let data = ImageData { img_type, data };
            map.serialize_field("image", &data)?;
        }

        map.serialize_field("text", &self.body)?;
        map.end()
    }
}

/// Abstraction over a forms element.
#[derive(Debug)]
pub enum Content {
    /// See [`Label`].
    Label(Label),
    /// See [`Input`].
    Input(Input),
    /// See [`Toggle`].
    Toggle(Toggle),
    /// See [`Dropdown`].
    Dropdown(Dropdown),
    /// See [`Slider`].
    Slider(Slider),
    /// See [`StepSlider`].
    StepSlider(StepSlider),
    // /// See [`Button`].
    // Button(Button),
}

impl serde::Serialize for Content {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            // Self::Button(b) => b.serialize(serializer),
            Self::Dropdown(d) => d.serialize(serializer),
            Self::Input(i) => i.serialize(serializer),
            Self::Label(l) => l.serialize(serializer),
            Self::Slider(s) => s.serialize(serializer),
            Self::StepSlider(s) => s.serialize(serializer),
            Self::Toggle(t) => t.serialize(serializer),
        }
    }
}
