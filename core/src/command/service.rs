use std::{sync::{Arc}, time::Duration};

use anyhow::Context;
use parking_lot::RwLock;
use proto::bedrock::CommandRequest;
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use util::Joinable;

const SERVICE_TIMEOUT: Duration = Duration::from_millis(10);

/// A response received from the command [`Service`].
#[derive(Debug)]
pub struct ServiceResponse {
    
}

/// A request that can be sent to the command [`Service`].
pub struct ServiceRequest {
    callback: oneshot::Sender<ServiceResponse>
}

/// Type used to communicate with the command [`Service`].
pub struct ServiceEndpoint {
    sender: mpsc::Sender<ServiceRequest>
}

impl ServiceEndpoint {
    /// Request execution of a command.
    /// 
    /// This method will return a receiver that will receive the output when the command has been executed.
    /// Execution of the command might not happen within the same tick.
    pub async fn request(&self, _request: CommandRequest<'_>) -> anyhow::Result<oneshot::Receiver<ServiceResponse>> {
        let (sender, receiver) = oneshot::channel();
        let request = ServiceRequest { callback: sender };

        self.sender.send_timeout(request, SERVICE_TIMEOUT).await.context("Command service request timed out")?;

        Ok(receiver)
    }
}

/// Service that manages command execution.
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
            token, handle: RwLock::new(None), sender
        });

        let clone = Arc::clone(&service);
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