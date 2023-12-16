//! Contains traits and structs of all agents.

pub mod assistant;
pub mod user_proxy;

use std::{error::Error, sync::Arc};

use tokio::sync::mpsc;

/// Messages exchanged between agents.
///
/// # Example
/// ```
/// use autogen::{agent::AgentMessage, builder::UserProxyBuilder};
///
/// let mut user = UserProxyBuilder::new("user_proxy")
///     .config_list(config_list.clone())
///     .build();
///
/// let mut another_user = UserProxyBuilder::new("user_proxy")
///     .config_list(config_list.clone())
///     .build();
///
/// let recipient = another_user.ctx.clone();
///
/// user.ctx
///     .tx
///     .send(AgentMessage::InitiateChat {
///         recipient,
///         request_reply: true,
///         message: "What is 1 + 1?",
///     })
///     .await
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub enum AgentMessage<'a> {
    InitiateChat {
        /// The agent to reply to.
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    },
    Send {
        /// The agent to reply to.
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    },
    Receive {
        /// The agent to reply to.
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    },
    Reset,
}

/// Wrapper around the name of an agent.
///
/// When calling "register_repply" it expects the name of the agent as a trigger
/// for the given callback.
///
/// With the wrapper we can ensure type safety and derive methods.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgentReplyTrigger<'a>(pub &'a str);

/// Context/public data of an Agent that is shared between other agents.
/// They can communicate by sending messages to `tx`.
#[derive(Debug, Clone)]
pub struct AgentCtx<'a> {
    pub name: &'a str,
    pub tx: mpsc::Sender<AgentMessage<'a>>,
}

/// Trait for building agents.
pub trait Agent<'a> {
    const DEFAULT_MODEL: &'a str = "gpt-4";
    const MAX_CONSECUTIVE_AUTO_REPLY: u32 = 100;

    /// Implementers of Agent must overwride this fn.
    fn run(&mut self) -> impl std::future::Future<Output = ()> + Send;
    fn generate_reply(
        &self,
    ) -> impl std::future::Future<Output = Option<String>>;
    fn send(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) -> impl std::future::Future<Output = ()> + Send;
    fn receive(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn Error>>> + Send;
}
