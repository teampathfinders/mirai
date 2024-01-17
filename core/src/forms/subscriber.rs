use std::{
    collections::HashMap,
    ops::Index,
    sync::atomic::{AtomicU32, Ordering},
};

use anyhow::{anyhow, Context};
use dashmap::DashMap;
use proto::bedrock::{FormCancelReason, FormRequest, FormResponseData};
use tokio::sync::oneshot;

use crate::{forms::FormElement, network::BedrockUser};

use super::{Form, FormDescriptor, Submittable};

#[derive(Debug)]
pub enum FormBodyValue {
    Bool(bool),
    Text(String),
    Index(u64),
    Float(f64),
}

impl FormBodyValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct FormBody {
    pub body: HashMap<String, FormBodyValue>,
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
        match self {
            Self::Cancelled(_) => true,
            _ => false,
        }
    }

    /// Returns the reason the form was cancelled.
    ///
    /// This function returns `None` if the form was not cancelled.
    #[inline]
    pub fn cancel_reason(&self) -> Option<FormCancelReason> {
        if let Self::Cancelled(reason) = self {
            Some(*reason)
        } else {
            None
        }
    }

    /// Returns the response of the form.
    ///
    /// This function returns `None` if the form did not receive a response.
    #[inline]
    pub fn response(&self) -> Option<&FormBody> {
        if let Self::Response(response) = self {
            Some(response)
        } else {
            None
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
    pub fn subscribe<F: Form>(&self, user: &BedrockUser, form: F) -> anyhow::Result<oneshot::Receiver<FormResponse>> {
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

        let body = response.response_data.ok_or_else(|| anyhow!("Form response body was empty"))?;

        let responses: serde_json::Value = serde_json::from_str(body).context("Unable to parse form response")?;

        let responses = responses.as_array().ok_or_else(|| anyhow!("Expected array response body"))?;

        let mut out = FormBody::default();
        let zip = std::iter::zip(desc, responses);
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

        let _ = sender.send(FormResponse::Response(out));
        Ok(())
    }
}
