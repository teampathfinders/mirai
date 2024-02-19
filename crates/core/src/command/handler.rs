use std::sync::Arc;

use proto::bedrock::Command;
use util::CowString;

use crate::{instance::Instance, net::BedrockClient};

use super::{ParseResult, ParsedCommand};

/// Represents a single output message in the command service response.
#[derive(Debug)]
pub struct HandlerOutput {
    /// Output of the command.
    pub message: CowString<'static>,
    // pub message: Cow<'static, str>
    /// Optional parameters used in the command output.
    // pub parameters: Vec<Cow<'static, str>>
    pub parameters: Vec<CowString<'static>>
}

impl HandlerOutput {
    /// Creates a new empty output.
    #[inline]
    pub const fn new() -> HandlerOutput {
        Self {
            message: CowString::empty(),
            parameters: Vec::new()
        }
    }

    /// Sets the message of this output.
    pub fn message<C: Into<CowString<'static>>>(mut self, message: C) -> HandlerOutput {
        self.message = message.into();
        self
    }

    /// Adds another parameter to the output.
    pub fn param<C: Into<CowString<'static>>>(mut self, param: C) -> HandlerOutput {
        self.parameters.push(param.into());
        self
    }

    /// Turns this into a successful message.
    #[inline]
    pub const fn success(self) -> HandlerResult {
        Ok(self)
    }

    /// Turns this into a command error.
    #[inline]
    pub const fn error(self) -> HandlerResult {
        Err(self)
    }
}

impl Default for HandlerOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of a command execution.
pub type HandlerResult = Result<HandlerOutput, HandlerOutput>;

/// Contains the caller of this command and the server instance.
pub struct Context {
    /// User that executed this command.
    pub caller: Arc<BedrockClient>,
    /// Access to all server data.
    pub instance: Arc<Instance>
}

/// A function that parses and executes a command.
pub trait CommandHandler: Send + Sync {
    /// Executes the command using this handler.
    /// This function also performs parsing of the input.
    fn call(&self, input: &str, ctx: &Context) -> HandlerResult;
    /// Returns the syntactic structure of the command.
    fn structure(&self) -> &Command;
}

/// A handler that uses the built-in command parser.
pub struct HandlerImpl<F> 
where
    F: Fn(ParsedCommand, &Context) -> HandlerResult + Send + Sync
{
    pub(super) handler: F,
    pub(super) structure: Command,
}

impl<F> CommandHandler for HandlerImpl<F> 
where
    F: Fn(ParsedCommand, &Context) -> HandlerResult + Send + Sync
{
    fn call(&self, input: &str, ctx: &Context) -> HandlerResult {
        // Parse command with default parser.
        let parsed = match ParsedCommand::default_parser(&self.structure, input) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(HandlerOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed, ctx)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}

/// A handler that uses a custom user-provided parser.
pub struct ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand, &Context) -> HandlerResult + Send + Sync,
    P: Fn(&str, &Context) -> ParseResult + Send + Sync
{
    pub(super) handler: F,
    pub(super) parser: P,
    pub(super) structure: Command
}

impl<F, P> CommandHandler for ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand, &Context) -> HandlerResult + Send + Sync,
    P: Fn(&str, &Context) -> ParseResult + Send + Sync
{
    fn call(&self, input: &str, ctx: &Context) -> HandlerResult {
        // Parse command with a custom parser.
        let parsed = match (self.parser)(input, ctx) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(HandlerOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed, ctx)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}