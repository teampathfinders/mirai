use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::anyhow;
use dashmap::DashMap;
use proto::bedrock::{FormRequest, FormResponse, FormCancelReason};
use tokio::sync::oneshot;

use crate::network::BedrockUser;

use super::Form;

pub struct FormResponseData {
    pub data: Option<String>,
    pub cancel_reason: Option<FormCancelReason>
}

impl FormResponseData {
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancel_reason.is_some()
    }
}

pub struct FormSubscriber {
    next_id: AtomicU32,   
    responders: DashMap<u32, oneshot::Sender<FormResponseData>>
}

impl FormSubscriber {
    /// Creates a new subscriber.
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(0), responders: DashMap::new()
        }
    }

    /// Submits a form to the user and returns a receiver that will receive the response.
    pub fn subscribe<F: Form>(&self, user: &BedrockUser, form: &F) -> anyhow::Result<oneshot::Receiver<FormResponseData>> {
        let data = serde_json::to_string(&form)?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        user.send(FormRequest { data: &data, id })?;

        let (sender, receiver) = oneshot::channel();
        self.responders.insert(id, sender);
        
        Ok(receiver)
    }

    pub fn handle_response(&self, response: FormResponse) -> anyhow::Result<()> {
        let (id, sender) = self.responders
            .remove(&response.id)
            .ok_or_else(|| anyhow!("Unregistered form response received. Please use the FormSubscriber interface instead of sending form requests directly"))?;

        let _  = sender.send(FormResponseData {
            data: response.response_data.map(String::from),
            cancel_reason: response.cancel_reason
        });

        Ok(())
    }
}