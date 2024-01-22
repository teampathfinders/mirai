use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::atomic::{AtomicU32, Ordering},
};

use anyhow::{anyhow, Context};
use dashmap::DashMap;
use proto::bedrock::{FormCancelReason, FormRequest, FormResponseData};
use tokio::sync::oneshot;

use crate::{
    forms::{FormElement, FormVariant},
    network::BedrockUser,
};

use super::{FormDescriptor, SubmittableForm};

/// A value that can be found in a custom form response.
#[derive(Debug)]
pub enum FormBodyValue {
    Bool(bool),
    Text(String),
    Index(u64),
    Float(f64),
}

impl FormBodyValue {
    /// Cast to a boolean.
    pub fn as_bool(&self) -> anyhow::Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => anyhow::bail!("Element response was not a bool"),
        }
    }

    /// Cast to a string.
    pub fn as_str(&self) -> anyhow::Result<&str> {
        match self {
            Self::Text(s) => Ok(s),
            _ => anyhow::bail!("Element response was not a string"),
        }
    }

    /// Cast to an integer.
    pub fn as_index(&self) -> anyhow::Result<u64> {
        match self {
            Self::Index(i) => Ok(*i),
            _ => anyhow::bail!("Element response was not an index"),
        }
    }

    /// Cast to a float.
    pub fn as_float(&self) -> anyhow::Result<f64> {
        match self {
            Self::Float(f) => Ok(*f),
            _ => anyhow::bail!("Element response was not a float"),
        }
    }
}

#[derive(Debug)]
pub enum FormBody {
    Modal(ModalFormBody),
    Menu(MenuFormBody),
    Custom(CustomFormBody),
}

impl FormBody {
    /// Casts this to a custom response.
    pub fn as_custom(&self) -> anyhow::Result<&CustomFormBody> {
        if let Self::Custom(body) = self {
            Ok(body)
        } else {
            anyhow::bail!("Form body did not come from a custom form")
        }
    }

    /// Casts this to a menu response.
    pub fn as_menu(&self) -> anyhow::Result<&MenuFormBody> {
        if let Self::Menu(body) = self {
            Ok(body)
        } else {
            anyhow::bail!("Form body did not come from a custom form")
        }
    }

    /// Casts this to a modal response.
    pub fn as_modal(&self) -> anyhow::Result<&ModalFormBody> {
        if let Self::Modal(body) = self {
            Ok(body)
        } else {
            anyhow::bail!("Form body did not come from a custom form")
        }
    }

    /// Whether this is a custom response.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Whether this is a menu response.
    pub fn is_menu(&self) -> bool {
        matches!(self, Self::Menu(_))
    }

    /// Whether this is a modal response.
    pub fn is_modal(&self) -> bool {
        matches!(self, Self::Modal(_))
    }
}

/// Response body of a [`ModalForm`].
#[derive(Debug)]
pub struct ModalFormBody {
    /// Whether the first button was pressed.
    pub confirmed: bool,
}

/// Response body of a [`MenuForm`].
#[derive(Debug)]
pub struct MenuFormBody {}

/// Response body of a [`CustomForm`].
#[derive(Debug, Default)]
pub struct CustomFormBody {
    /// Form body
    body: HashMap<String, FormBodyValue>,
}

impl CustomFormBody {
    pub fn get(&self, index: impl AsRef<str>) -> Option<&FormBodyValue> {
        self.body.get(index.as_ref())
    }

    pub fn get_mut(&mut self, index: impl AsRef<str>) -> Option<&mut FormBodyValue> {
        self.body.get_mut(index.as_ref())
    }
}

impl<S: AsRef<str>> Index<S> for CustomFormBody {
    type Output = FormBodyValue;

    fn index(&self, index: S) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<S: AsRef<str>> IndexMut<S> for CustomFormBody {
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

/// A type of response to a form.
#[derive(Debug)]
pub enum FormResponse {
    /// A response was received.
    Response(FormBody),
    /// The form was cancelled.
    Cancelled(FormCancelReason),
}

impl FormResponse {
    /// Whether the form was cancelled.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(_))
    }

    /// Casts to a [`FormCancelReason`].
    ///
    /// Returns an error if the form was not cancelled.
    #[inline]
    pub fn as_cancelled(&self) -> anyhow::Result<FormCancelReason> {
        if let Self::Cancelled(reason) = self {
            Ok(*reason)
        } else {
            anyhow::bail!("Form response was not cancelled")
        }
    }

    /// Casts to a [`FormBody`].
    ///
    /// Returns an error if the form was cancelled.
    #[inline]
    pub fn as_response(&self) -> anyhow::Result<&FormBody> {
        if let Self::Response(response) = self {
            Ok(response)
        } else {
            anyhow::bail!("Form response was cancelled")
        }
    }
}

#[derive(Debug)]
pub struct FormSubscriber {
    next_id: AtomicU32,
    subscribed: DashMap<u32, (oneshot::Sender<FormResponse>, FormDescriptor)>,
}

impl FormSubscriber {
    /// Creates a new subscriber.
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(0),
            subscribed: DashMap::new(),
        }
    }

    /// Submits a form to the user and returns a receiver that will receive the response.
    pub fn subscribe<F: SubmittableForm>(&self, user: &BedrockUser, form: F) -> anyhow::Result<oneshot::Receiver<FormResponse>> {
        let data = serde_json::to_string(&form)?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        user.send(FormRequest { data: &data, id })?;

        let (sender, receiver) = oneshot::channel();
        self.subscribed.insert(id, (sender, form.into_descriptor()));

        Ok(receiver)
    }

    pub fn handle_response(&self, response: FormResponseData) -> anyhow::Result<()> {
        let (_id, (sender, desc)) = self.subscribed.remove(&response.id).ok_or_else(|| {
            anyhow!("Unregistered form response received. Please use the FormSubscriber interface instead of sending form requests directly")
        })?;

        if let Some(reason) = response.cancel_reason {
            let _ = sender.send(FormResponse::Cancelled(reason));
            return Ok(());
        }

        tracing::info!("{response:?}");

        let body = response.response_data.ok_or_else(|| anyhow!("Form response body was empty"))?;
        match desc.variant {
            FormVariant::Custom => self.handle_custom(desc, sender, body),
            FormVariant::Modal => self.handle_modal(desc, sender, body),
            FormVariant::Menu => self.handle_menu(desc, sender, body),
        }
    }

    fn handle_menu(&self, desc: FormDescriptor, sender: oneshot::Sender<FormResponse>, body: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_modal(&self, desc: FormDescriptor, sender: oneshot::Sender<FormResponse>, body: &str) -> anyhow::Result<()> {
        let confirmed: bool = serde_json::from_str(body).context("Unable to parse modal response")?;

        // If this returns an error, then the receiver was dropped which can be ignored.
        let _ = sender.send(FormResponse::Response(FormBody::Modal(ModalFormBody { confirmed })));

        Ok(())
    }

    fn handle_custom(&self, desc: FormDescriptor, sender: oneshot::Sender<FormResponse>, body: &str) -> anyhow::Result<()> {
        let responses: Vec<serde_json::Value> = serde_json::from_str(body).context("Unable to parse custom form response")?;

        let mut out = CustomFormBody::default();
        let zip = std::iter::zip(desc.content, responses);
        for ((key, desc), res) in zip {
            match desc {
                FormElement::Label(_) => {
                    // Minecraft also sends a null response for label elements.
                    if !res.is_null() {
                        anyhow::bail!("Received non-null response for label")
                    }
                }
                FormElement::Toggle(_) => {
                    let res = res.as_bool().ok_or_else(|| anyhow!("Expected toggle response to be a boolean"))?;

                    out.body.insert(key, FormBodyValue::Bool(res));
                }
                FormElement::Input(_) => {
                    let res = res.as_str().ok_or_else(|| anyhow!("Expected input response to be a string"))?;

                    out.body.insert(key, FormBodyValue::Text(res.to_owned()));
                }
                FormElement::Dropdown(dropdown) => {
                    let res = res.as_u64().ok_or_else(|| anyhow!("Expected dropdown response to be an integer"))?;

                    let max_allowed = dropdown.options.len() as u64;
                    if res >= max_allowed {
                        anyhow::bail!("Dropdown option out of range ({res} >= {max_allowed})")
                    }

                    out.body.insert(key, FormBodyValue::Index(res));
                }
                FormElement::Slider(slider) => {
                    let res = res.as_f64().ok_or_else(|| anyhow!("Expected slider response to be a float"))?;

                    if res < slider.min {
                        anyhow::bail!("Slider input out of range ({res} < {})", slider.min);
                    }

                    if res > slider.max {
                        anyhow::bail!("Slider input out of range ({res} > {})", slider.max);
                    }

                    if (res / slider.step).fract() != 0.0 {
                        anyhow::bail!("Slider input does not match specified step");
                    }

                    out.body.insert(key, FormBodyValue::Float(res));
                }
                FormElement::StepSlider(slider) => {
                    let res = res.as_u64().ok_or_else(|| anyhow!("Expected step slider response to be an integer"))?;

                    let max_allowed = slider.steps.len() as u64;
                    if res >= max_allowed {
                        anyhow::bail!("Step slider option out of range ({res} >= {max_allowed})");
                    }

                    out.body.insert(key, FormBodyValue::Index(res));
                }
                _ => anyhow::bail!("Invalid form element descriptor"),
            }
        }

        // If this returns an error, then the receiver was dropped which can be ignored.
        let _ = sender.send(FormResponse::Response(FormBody::Custom(out)));

        Ok(())
    }
}

impl Default for FormSubscriber {
    fn default() -> Self {
        Self::new()
    }
}
