use std::{sync::{Arc}, time::Duration};

use anyhow::Context;
use parking_lot::RwLock;
use proto::bedrock::CommandRequest;
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use util::Joinable;

const SERVICE_TIMEOUT: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct ServiceResponse {
    
}

pub struct ServiceRequest {
    callback: oneshot::Sender<ServiceResponse>
}

pub struct ServiceEndpoint {
    sender: mpsc::Sender<ServiceRequest>
}

impl ServiceEndpoint {
    pub async fn request(&self, _request: CommandRequest<'_>) -> anyhow::Result<oneshot::Receiver<ServiceResponse>> {
        let (sender, receiver) = oneshot::channel();
        let request = ServiceRequest { callback: sender };

        self.sender.send_timeout(request, SERVICE_TIMEOUT).await.context("Command service request timed out")?;

        Ok(receiver)
    }
}

pub struct Service {
    token: CancellationToken,
    handle: RwLock<Option<JoinHandle<()>>>,

    /// Simply stored here so it can be used for endpoints.
    sender: mpsc::Sender<ServiceRequest>
}

impl Service {
    /// Creates a new command service.
    pub fn new(token: CancellationToken) -> Arc<Service> {
        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(Service {
            token, handle: RwLock::new(None), sender: sender.clone()
        });

        let clone = service.clone();
        let handle = tokio::spawn(async move {
            clone.execution_job(receiver).await
        });

        *service.handle.write() = Some(handle);
        service
    }

    /// Creates a new [`ServiceEndpoint`].
    pub fn create_endpoint(&self) -> ServiceEndpoint {
        ServiceEndpoint { sender: self.sender.clone() }
    }

    /// Runs the service execution job.
    async fn execution_job(self: Arc<Service>, mut receiver: mpsc::Receiver<ServiceRequest>) {
        // loop {
            tokio::select! {
                _ = self.token.cancelled() => {
                    // Stop accepting requests.
                    receiver.close();
                    // break
                }   
            }
        // }

        tracing::info!("Command service closed");
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        let handle = self.handle.write().take();
        match handle {
            Some(handle) => Ok(handle.await?),
            None => anyhow::bail!("This command service has already been joined.")
        }
    }
}