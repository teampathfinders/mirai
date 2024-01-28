//! Utilities for handling form responses.

use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::atomic::{AtomicU32, Ordering},
};

use anyhow::{anyhow, Context};
use dashmap::DashMap;
use proto::bedrock::{CancelReason, FormRequest, FormResponseData};
use tokio::sync::oneshot;

use crate::{forms::Content, net::BedrockUser};

use super::{FormDesc, SubmittableForm};

/// A value that can be found in a custom form response.
#[derive(Debug)]
pub enum BodyValue {
    Bool(bool),
    Text(String),
    Index(u64),
    Float(f64),
}

impl BodyValue {
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

/// A general form response body.
#[derive(Debug)]
pub enum Body {
    /// A response to a modal form.
    Modal(ModalResponse),
    /// A response to a menu form.
    Menu(MenuResponse),
    /// A response to a custom form.
    Custom(CustomResponse),
}

impl Body {
    /// Casts this to a custom response.
    pub fn as_custom(&self) -> anyhow::Result<&CustomResponse> {
        if let Self::Custom(body) = self {
            Ok(body)
        } else {
            anyhow::bail!("Form body did not come from a custom form")
        }
    }

    /// Casts this to a menu response.
    pub fn as_menu(&self) -> anyhow::Result<&MenuResponse> {
        if let Self::Menu(body) = self {
            Ok(body)
        } else {
            anyhow::bail!("Form body did not come from a custom form")
        }
    }

    /// Casts this to a modal response.
    pub fn as_modal(&self) -> anyhow::Result<&ModalResponse> {
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

/// Response body of a [`Modal`].
#[derive(Debug)]
pub struct ModalResponse {
    /// Whether the first button was pressed.
    confirmed: bool,
}

impl ModalResponse {
    /// Whether the confirm button was pressed.
    pub fn confirmed(&self) -> bool {
        self.confirmed
    }
}

/// Response body of a [`Menu`].
#[derive(Debug)]
pub struct MenuResponse {
    /// Which button was pressed.
    pressed: usize,
}

impl MenuResponse {
    /// Which button was pressed. This corresponds to the nth button added to the form.
    /// Index starts at 0.
    pub fn pressed(&self) -> usize {
        self.pressed
    }
}

/// Response body of a [`Custom`] form.
#[derive(Debug, Default)]
pub struct CustomResponse {
    /// Form body
    body: HashMap<String, BodyValue>,
}

impl CustomResponse {
    /// Gets a shared reference to the item at the given key.
    pub fn get(&self, index: impl AsRef<str>) -> Option<&BodyValue> {
        self.body.get(index.as_ref())
    }

    /// Gets a mutable reference to the item at the given key.
    pub fn get_mut(&mut self, index: impl AsRef<str>) -> Option<&mut BodyValue> {
        self.body.get_mut(index.as_ref())
    }
}

impl<S: AsRef<str>> Index<S> for CustomResponse {
    type Output = BodyValue;

    fn index(&self, index: S) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<S: AsRef<str>> IndexMut<S> for CustomResponse {
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

/// A type of response to a form.
#[derive(Debug)]
pub enum Response {
    /// A response was received.
    Body(Body),
    /// The form was cancelled.
    Cancelled(CancelReason),
}

impl Response {
    /// Whether the form was cancelled.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(_))
    }

    /// Casts to a [`FormCancelReason`].
    ///
    /// Returns an error if the form was not cancelled.
    #[inline]
    pub fn as_cancelled(&self) -> anyhow::Result<CancelReason> {
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
    pub fn as_body(&self) -> anyhow::Result<&Body> {
        if let Self::Body(response) = self {
            Ok(response)
        } else {
            anyhow::bail!("Form response was cancelled")
        }
    }
}

/// Listens for responses to forms.
///
/// Create a form and add it to the subscriber by calling the [`subscribe`](Subscriber::subscribe) method.
/// This method then returns a channel which you can use to await the response.
#[derive(Debug)]
pub struct Subscriber {
    next_id: AtomicU32,
    subscribed: DashMap<u32, (oneshot::Sender<Response>, FormDesc)>,
}

impl Subscriber {
    /// Creates a new subscriber.
    pub(crate) fn new() -> Self {
        Self {
            next_id: AtomicU32::new(0),
            subscribed: DashMap::new(),
        }
    }

    /// Submits a form to the user and returns a receiver that will receive the response.
    pub fn subscribe<F: SubmittableForm>(&self, user: &BedrockUser, form: F) -> anyhow::Result<oneshot::Receiver<Response>> {
        let data = serde_json::to_string(&form)?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        user.send(FormRequest { data: &data, id })?;

        let (sender, receiver) = oneshot::channel();
        self.subscribed.insert(id, (sender, form.into_desc()));

        Ok(receiver)
    }

    /// Handles a form response.
    pub(crate) fn handle_response(&self, response: FormResponseData) -> anyhow::Result<()> {
        let (_id, (sender, desc)) = self.subscribed.remove(&response.id).ok_or_else(|| {
            anyhow!("Unregistered form response received. Please use the FormSubscriber interface instead of sending form requests directly")
        })?;

        if let Some(reason) = response.cancel_reason {
            let _ = sender.send(Response::Cancelled(reason));
            return Ok(());
        }

        let body = response.response_data.ok_or_else(|| anyhow!("Form response body was empty"))?;

        match desc {
            FormDesc::Custom(desc) => self.handle_custom(desc, sender, body),
            FormDesc::Modal => self.handle_modal(sender, body),
            FormDesc::Menu => self.handle_menu(sender, body),
        }
    }

    /// Handles a menu response.
    fn handle_menu(&self, sender: oneshot::Sender<Response>, body: &str) -> anyhow::Result<()> {
        let pressed: usize = serde_json::from_str(body).context("Unable to parse menu response")?;

        // If this returns an error, then the receiver was dropped which can be ignored.
        let _ = sender.send(Response::Body(Body::Menu(MenuResponse { pressed })));

        Ok(())
    }

    /// Handles a modal response.
    fn handle_modal(&self, sender: oneshot::Sender<Response>, body: &str) -> anyhow::Result<()> {
        let confirmed: bool = serde_json::from_str(body).context("Unable to parse modal response")?;

        // If this returns an error, then the receiver was dropped which can be ignored.
        let _ = sender.send(Response::Body(Body::Modal(ModalResponse { confirmed })));

        Ok(())
    }

    /// Handles a custom response.
    fn handle_custom(&self, desc: HashMap<String, Content>, sender: oneshot::Sender<Response>, body: &str) -> anyhow::Result<()> {
        let responses: Vec<serde_json::Value> = serde_json::from_str(body).context("Unable to parse custom form response")?;

        let mut out = CustomResponse::default();
        let zip = std::iter::zip(desc, responses);
        for ((key, desc), res) in zip {
            match desc {
                Content::Label(_) => {
                    // Minecraft also sends a null response for label elements.
                    if !res.is_null() {
                        anyhow::bail!("Received non-null response for label")
                    }
                }
                Content::Toggle(_) => {
                    let res = res.as_bool().ok_or_else(|| anyhow!("Expected toggle response to be a boolean"))?;

                    out.body.insert(key, BodyValue::Bool(res));
                }
                Content::Input(_) => {
                    let res = res.as_str().ok_or_else(|| anyhow!("Expected input response to be a string"))?;

                    out.body.insert(key, BodyValue::Text(res.to_owned()));
                }
                Content::Dropdown(dropdown) => {
                    let res = res.as_u64().ok_or_else(|| anyhow!("Expected dropdown response to be an integer"))?;

                    let max_allowed = dropdown.options.len() as u64;
                    if res >= max_allowed {
                        anyhow::bail!("Dropdown option out of range ({res} >= {max_allowed})")
                    }

                    out.body.insert(key, BodyValue::Index(res));
                }
                Content::Slider(slider) => {
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

                    out.body.insert(key, BodyValue::Float(res));
                }
                Content::StepSlider(slider) => {
                    let res = res.as_u64().ok_or_else(|| anyhow!("Expected step slider response to be an integer"))?;

                    let max_allowed = slider.steps.len() as u64;
                    if res >= max_allowed {
                        anyhow::bail!("Step slider option out of range ({res} >= {max_allowed})");
                    }

                    out.body.insert(key, BodyValue::Index(res));
                }
            }
        }

        // If this returns an error, then the receiver was dropped which can be ignored.
        let _ = sender.send(Response::Body(Body::Custom(out)));

        Ok(())
    }
}

impl Default for Subscriber {
    fn default() -> Self {
        Self::new()
    }
}
