use std::{sync::atomic::{AtomicU32, Ordering}, ops::Index, collections::HashMap};

use anyhow::anyhow;
use dashmap::DashMap;
use proto::bedrock::{FormRequest, FormResponseData, FormCancelReason};
use tokio::sync::oneshot;

use crate::network::BedrockUser;

use super::{Form, Submittable, FormDescriptor};

pub enum FormResponseBodyElement {
    Test
}

impl FormResponseBodyElement {
    pub fn cast<'a, T: Submittable>(self) -> Option<T> {
        todo!()
    }
}

#[derive(Debug)]
pub struct FormResponseBody {

}

impl Index<&str> for FormResponseBody {
    type Output = FormResponseBodyElement;

    fn index(&self, key: &str) -> &Self::Output {
        &FormResponseBodyElement::Test
    }
}

#[derive(Debug)]
pub enum FormResponse {
    Response(FormResponseBody),
    Cancelled(FormCancelReason)
}

impl FormResponse {
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        match self {
            Self::Cancelled(_) => true,
            _ => false
        }
    }

    #[inline]
    pub fn cancel_reason(&self) -> Option<FormCancelReason> {
        if let Self::Cancelled(reason) = self { Some(*reason) } else { None }
    }

    #[inline]
    pub fn response_body(&self) -> Option<&FormResponseBody> {
        if let Self::Response(response) = self { Some(response) } else { None }
    }
}

#[derive(Debug)]
pub struct FormSubscriber {
    next_id: AtomicU32,   
    subscribed: DashMap<u32, (oneshot::Sender<FormResponse>, FormDescriptor)>
}

impl FormSubscriber {
    /// Creates a new subscriber.
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(0), subscribed: DashMap::new()
        }
    }

    /// Submits a form to the user and returns a receiver that will receive the response.
    pub fn subscribe<F: Form>(&self, user: &BedrockUser, form: F) -> anyhow::Result<oneshot::Receiver<FormResponse>> {
        let data = serde_json::to_string(&form)?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        tracing::debug!("{data}");

        user.send(FormRequest { data: &data, id })?;

        let (sender, receiver) = oneshot::channel();
        self.subscribed.insert(id, (sender, form.into_descriptor()));
        
        Ok(receiver)
    }

    pub fn handle_response(&self, response: FormResponseData) -> anyhow::Result<()> {
        let (_id, (sender, desc)) = self.subscribed
            .remove(&response.id)
            .ok_or_else(|| anyhow!("Unregistered form response received. Please use the FormSubscriber interface instead of sending form requests directly"))?;

        if let Some(reason) = response.cancel_reason {
            let _ = sender.send(FormResponse::Cancelled(reason));
            return Ok(())
        }

        todo!();
        // let _  = sender.send(FormResponseData {
        //     data: response.response_data.map(String::from),
        //     cancel_reason: response.cancel_reason
        // });

        Ok(())
    }
}