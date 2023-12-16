//! Agent that communicate with user to get prompts and execute code.
use std::{error::Error, sync::Arc};

use hashbrown::HashMap;
use tokio::sync::mpsc;

use crate::config::Config;

use super::{Agent, AgentCtx, AgentMessage};

pub struct UserProxy<'a> {
    pub ctx: Arc<AgentCtx<'a>>,
    pub config_list: Vec<Config>,
    pub messages: HashMap<&'a str, Vec<String>>,
    pub rx: Option<mpsc::Receiver<AgentMessage<'a>>>,
}

impl<'a> UserProxy<'a> {
    pub fn new(
        name: &'a str,
        tx: mpsc::Sender<AgentMessage<'a>>,
        rx: mpsc::Receiver<AgentMessage<'a>>,
        config_list: Vec<Config>,
    ) -> Self {
        let ctx = Arc::new(AgentCtx { name, tx });
        Self { ctx, rx: Some(rx), config_list, messages: HashMap::new() }
    }
    /// The very first message of the chat. Prepares initial logic, and then
    /// `send` the message.
    async fn initiate_chat(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) {
        // self.prepare_chat();
        self.send(recipient, message.to_owned(), request_reply).await;
    }
    /// Stores the received message and loggs it.
    fn process_received_message(
        &mut self,
        recipient: &Arc<AgentCtx<'a>>,
        message: String,
    ) {
        println!("{} received message: {message}", self.ctx.name);
        let messages = self.messages.entry(&recipient.name).or_default();
        messages.push(message);
    }
}

impl<'a> Agent<'a> for UserProxy<'a> {
    /// Spawns the event loop of `self.rx`, listen to messages sent by other
    /// agents.
    async fn run(&mut self) {
        let mut rx = std::mem::take(&mut self.rx).unwrap();

        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                Send { message, recipient, request_reply } => {
                    self.send(recipient, message.to_owned(), request_reply)
                        .await;
                }
                InitiateChat { recipient, message, request_reply } => {
                    self.initiate_chat(recipient, message, request_reply).await;
                }
                Receive { recipient, message, request_reply } => {
                    let _ =
                        self.receive(recipient, message, request_reply).await;
                }
                _ => {}
            }
        }
    }
    /// Sends a message to another agent.
    async fn send(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) {
        self.process_received_message(&recipient, message.clone());
        let _ = recipient
            .tx
            .send(AgentMessage::Receive {
                recipient: self.ctx.clone(),
                message,
                request_reply,
            })
            .await;
    }
    /// Once a message is received, this function sends a reply to the sender or
    /// stop.
    async fn receive(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) -> Result<(), Box<dyn Error>> {
        self.process_received_message(&recipient, message.to_owned());
        if !request_reply {
            return Ok(());
        };
        let reply = self.generate_reply().await;
        if let Some(reply) = reply {
            self.send(recipient, reply.to_owned(), request_reply).await;
        }
        Ok(())
    }
    async fn generate_reply(&self) -> Option<String> {
        Some("fake message".to_owned())
    }
}
