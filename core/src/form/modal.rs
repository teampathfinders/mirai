use serde::ser::SerializeStruct;

#[derive(Debug)]
pub struct FormLabel<'a> {
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

#[derive(Debug)]
pub struct FormInput<'a> {
    pub label: &'a str,
    pub default: &'a str,
    pub placeholder: &'a str,
}

impl<'a> serde::Serialize for FormInput<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("input", 4)?;
        map.serialize_field("type", "input")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", self.default)?;
        map.serialize_field("placeholder", self.placeholder)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct FormToggle<'a> {
    pub(crate) label: &'a str,
    pub(crate) default: bool,
}

impl<'a> serde::Serialize for FormToggle<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("toggle", 3)?;
        map.serialize_field("type", "toggle")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", &self.default)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct FormSlider<'a> {
    pub(crate) label: &'a str,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) step: f64,
    pub(crate) default: f64,
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
        map.serialize_field("default", &self.default)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct FormDropdown<'a> {
    pub(crate) label: &'a str,
    pub(crate) options: &'a [&'a str],
    pub(crate) default: i32,
}

impl<'a> serde::Serialize for FormDropdown<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("dropdown", 4)?;
        map.serialize_field("type", "dropdown")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", &self.default)?;
        map.serialize_field("options", self.options)?;
        map.end()
    }
}

#[derive(Debug)]
pub struct FormStepSlider<'a> {
    pub(crate) label: &'a str,
    pub(crate) steps: &'a [&'a str],
    pub(crate) default: i32,
}

impl<'a> serde::Serialize for FormStepSlider<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("step_slider", 4)?;
        map.serialize_field("type", "step_slider")?;
        map.serialize_field("text", self.label)?;
        map.serialize_field("default", &self.default)?;
        map.serialize_field("steps", self.steps)?;
        map.end()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FormButtonImage<'a> {
    Url(&'a str),
    Path(&'a str)
}

#[derive(Debug)]
pub struct FormButton<'a> {
    pub(crate) label: &'a str,
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

#[derive(Debug)]
pub enum FormElement<'a> {
    Label(FormLabel<'a>),
    Input(FormInput<'a>),
    Toggle(FormToggle<'a>),
    Dropdown(FormDropdown<'a>),
    Slider(FormSlider<'a>),
    StepSlider(FormStepSlider<'a>),
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