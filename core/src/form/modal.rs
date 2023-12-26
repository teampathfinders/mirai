// #[derive(Debug, Clone)]
// pub struct FormDropdown<'a> {
//     label: &'a str,
//     options: &'a [&'a str]
// }
//
// #[derive(Debug, Clone)]
// pub struct FormText<'a> {
//     label: &'a str,
//     placeholder: &'a str,
//     default: Option<&'a str>
// }
//
// #[derive(Debug, Clone)]
// pub struct FormToggle<'a> {
//     label: &'a str,
//     default: Option<bool>
// }
//
// #[derive(Debug, Clone)]
// pub struct FormSlider<'a> {
//     label: &'a str,
//     min: u32,
//     max: u32,
//     step: u32,
//     default: Option<u32>
// }
//
// #[derive(Debug, Clone)]
// pub enum FormChild<'a> {
//     Dropdown(FormDropdown<'a>),
//     Text(FormText<'a>),
//     Toggle(FormToggle<'a>),
//     Slider(FormSlider<'a>)
// }
//
// /// Builds form forms.
// #[derive(Debug, Clone)]
// pub struct FormBuilder<'a> {
//     title: &'a str,
//     children: Vec<FormChild<'a>>
// }
//
// impl<'a> FormBuilder<'a> {
//     /// Creates a new form form builder.
//     pub fn new() -> Self {
//         Self {
//             title: "Modal title",
//             children: Vec::new()
//         }
//     }
//
//     /// Sets the title of the form form.
//     #[inline]
//     pub fn title(&mut self, title: &'a str) -> &mut Self {
//         self.title = title;
//         self
//     }
//
//     #[inline]
//     pub fn dropdown(&mut self, label: &'a str, options: &'a [&'a str]) -> &mut Self {
//         self.children.push(FormChild::Dropdown(FormDropdown {
//             label, options
//         }));
//         self
//     }
//
//     #[inline]
//     pub fn text(&mut self, label: &'a str, placeholder: &'a str, default: Option<&'a str>) -> &mut Self {
//         self.children.push(FormChild::Text(FormText {
//             label, placeholder, default
//         }));
//         self
//     }
//
//     #[inline]
//     pub fn slider(&mut self, label: &'a str, min: u32, max: u32, step: u32, default: Option<u32>) -> &mut Self {
//         self.children.push(FormChild::Slider(FormSlider {
//             label, min, max, step, default
//         }));
//         self
//     }
//
//     #[inline]
//     pub fn toggle(&mut self, label: &'a str, default: Option<bool>) -> &mut Self {
//         self.children.push(FormChild::Toggle(FormToggle {
//             label, default
//         }));
//         self
//     }
//
//     /// Builds the form and consumes the builder.
//     pub fn build(self) -> Form<'a> {
//         Form {
//             title: self.title,
//             children: self.children
//         }
//     }
// }

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

// impl<'a> FormLabel<'a> {
//     pub fn to_json(&self) -> String {
//         serde_json::json!({
//             "type": "label",
//             "text": self.label
//         }).to_string()
//     }
// }

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

// impl<'a> FormInput<'a> {
//     pub fn to_json(&self) -> String {
//         serde_json::json!({
//             "type": "input",
//             "text": self.label,
//             "default": self.default,
//             "placeholder": self.placeholder
//         }).to_string()
//     }
// }

#[derive(Debug)]
pub struct FormToggle<'a> {
    label: &'a str,
    default: bool,
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

// impl<'a> FormToggle<'a> {
//     pub fn to_json(&self) -> String {
//         serde_json::json!({
//             "type": "toggle",
//             "text": self.label,
//             "default": self.default
//         }).to_string()
//     }
// }

#[derive(Debug)]
pub struct FormSlider<'a> {
    label: &'a str,
    min: f64,
    max: f64,
    step: f64,
    default: f64,
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

// impl<'a> FormSlider<'a> {
//     pub fn to_json(&self) -> String {
//         serde_json::json!({
//             "type": "slider",
//             "text": self.label,
//             "min": self.min,
//             "max": self.max,
//             "step": self.step,
//             "default": self.default
//         }).to_string()
//     }
// }

#[derive(Debug)]
pub struct FormDropdown<'a> {
    label: &'a str,
    options: &'a [&'a str],
    default: i32,
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

// impl<'a> FormDropdown<'a> {
//     pub fn to_json(&self) -> String {
//         serde_json::json!({
//             "type": "dropdown",
//             "text": self.label,
//             "default": self.default,
//             "options": self.options
//         }).to_string()
//     }
// }

#[derive(Debug)]
pub struct FormStepSlider<'a> {
    label: &'a str,
    steps: &'a [&'a str],
    default: i32,
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

// impl<'a> FormStepSlider<'a> {
//     pub fn to_json(&self) -> String {
//         serde_json::json!({
//             "type": "step_slider",
//             "text": self.label,
//             "default": self.default,
//             "steps": self.steps
//         }).to_string()
//     }
// }

#[derive(Debug)]
pub struct FormButton<'a> {
    label: &'a str,
    image: Option<&'a str>,
}

impl<'a> serde::Serialize for FormButton<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
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
        if let Some(data) = self.image {
            let img_type = if data.starts_with("http") { "url" } else { "path" };

            let data = ImageData { img_type, data };

            map.serialize_field("image", &data)?;
        } else {
            map.serialize_field("text", self.label)?;
        }
        map.end()
    }
}

// impl<'a> FormButton<'a> {
//     pub fn to_json(&self) -> String {
//         if let Some(image) = self.image {
//             let b_type = if image.starts_with("http") {
//                 "url"
//             } else {
//                 "path"
//             };
//
//             serde_json::json!({
//                 "image": {
//                     "type": b_type,
//                     "data": image
//                 }
//             }).to_string()
//         } else {
//             serde_json::json!({
//                 "text": self.label
//             }).to_string()
//         }
//     }
// }

#[derive(Debug)]
pub struct Menu<'a> {
    title: &'a str,
    content: &'a str,
    buttons: &'a [&'a str],
}

impl<'a> Menu<'a> {
    pub fn to_json(&self) -> String {
        let json = serde_json::json!({
            "type": "form",
            "title": self.title,
            "content": self.content,
            "buttons": self.buttons
        });
        json.to_string()
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
    pub elements: Vec<FormElement<'a>>,
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
        map.serialize_field("content", &self.elements[1])?;
        map.serialize_field("button1", self.button1)?;
        map.serialize_field("button2", self.button2)?;
        map.end()
    }
}

// impl<'a> Modal<'a> {
//     pub fn to_json(&self) -> String {
//         let json = serde_json::json!({
//             "type": "modal",
//             "title": self.title,
//             "content": self.content,
//             "button1": self.button1,
//             "button2": self.button2
//         });
//         json.to_string()
//     }
// }
