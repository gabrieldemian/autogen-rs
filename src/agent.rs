use std::{error::Error, sync::Arc};

use crate::config::Config;
use hashbrown::HashMap;
use tokio::sync::mpsc;

/// Messages exchanged between agents.
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

/// When defining a custom reply for a custom agent,
/// it can have the following triggers:
pub enum AgentReplyTrigger<'a> {
    /// Trigger by name of the calling agent.
    Name(&'a str),
    // todo: support implementations of Agent in the future.
}

pub type AgentReplyFn = Box<dyn FnMut() + std::marker::Send>;

/// Context/public data of an Agent that is shared between other agents.
/// They can communicate by sending messages to `tx`.
pub struct AgentCtx<'a> {
    pub name: &'a str,
    pub tx: mpsc::Sender<AgentMessage<'a>>,
}

pub struct UserProxy<'a> {
    pub ctx: Arc<AgentCtx<'a>>,
    pub config_list: Vec<Config>,
    pub messages: HashMap<&'a str, Vec<String>>,
    pub rx: Option<mpsc::Receiver<AgentMessage<'a>>>,
}

pub struct Assistant<'a> {
    pub ctx: Arc<AgentCtx<'a>>,
    pub config_list: Vec<Config>,
    pub messages: HashMap<&'a str, Vec<&'a str>>,
    pub rx: Option<mpsc::Receiver<AgentMessage<'a>>>,
    pub reply_fn_list: Vec<Box<dyn FnMut() + std::marker::Send>>,
}

/// All custom Agents must implement this trait.
pub trait Agent<'a> {
    const DEFAULT_MODEL: &'a str = "gpt-4";
    const MAX_CONSECUTIVE_AUTO_REPLY: u32 = 100;
    const DEFAULT_DESCRIPTION: &'a str = "";
    const DEFAULT_SYSTEM_MESSAGE: &'a str = "";

    /// Implementers of Agent must overwride this fn.
    fn run(&mut self) -> impl std::future::Future<Output = ()> + Send;
}

/// Trait for creating custom agents derived from `Assistant`.
pub trait AssistantAgent<'a> {
    fn register_repply(
        &mut self,
        trigger: AgentReplyTrigger<'a>,
        function: AgentReplyFn,
    );
}

// Blanked impl for Assistant, anyone who implements Assistant
// will automatically implement AssistantAgent.
impl<'a> AssistantAgent<'a> for Assistant<'a> {
    fn register_repply(
        &mut self,
        trigger: AgentReplyTrigger<'a>,
        function: AgentReplyFn,
    ) {
        //
    }
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
    fn generate_reply(&self) -> Option<&str> {
        Some("fake message")
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
        let reply = self.generate_reply();
        if let Some(reply) = reply {
            self.send(recipient, reply.to_owned(), request_reply).await;
        }
        Ok(())
    }
}

impl<'a> Agent<'a> for UserProxy<'a> {
    const DEFAULT_DESCRIPTION: &'a str = "A helpful and general-purpose AI assistant that has strong language skills, Python skills, and Linux command line skills.";
    const DEFAULT_SYSTEM_MESSAGE: &'a str = r#"You are a helpful AI assistant.
Solve tasks using your coding and language skills.
In the following cases, suggest python code (in a python coding block) or shell script (in a sh coding block) for the user to execute.
    1. When you need to collect info, use the code to output the info you need, for example, browse or search the web, download/read a file, print the content of a webpage or a file, get the current date/time, check the operating system. After sufficient info is printed and the task is ready to be solved based on your language skill, you can solve the task by yourself.
    2. When you need to perform some task with code, use the code to perform the task and output the result. Finish the task smartly.
Solve the task step by step if you need to. If a plan is not provided, explain your plan first. Be clear which step uses code, and which step uses your language skill.
When using code, you must indicate the script type in the code block. The user cannot provide any other feedback or perform any other action beyond executing the code you suggest. The user can't modify your code. So do not suggest incomplete code which requires users to modify. Don't use a code block if it's not intended to be executed by the user.
If you want the user to save the code in a file before executing it, put # filename: <filename> inside the code block as the first line. Don't include multiple code blocks in one response. Do not ask users to copy and paste the result. Instead, use 'print' function for the output when relevant. Check the execution result returned by the user.
If the result indicates there is an error, fix the error and output the code again. Suggest the full code instead of partial code or code changes. If the error can't be fixed or if the task is not solved even after the code is executed successfully, analyze the problem, revisit your assumption, collect additional info you need, and think of a different approach to try.
When you find an answer, verify the answer carefully. Include verifiable evidence in your response if possible.
Reply "TERMINATE" in the end when everything is done."#;
    const DEFAULT_MODEL: &'a str = "gpt-4";
    const MAX_CONSECUTIVE_AUTO_REPLY: u32 = 100;
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
}

impl<'a> Assistant<'a> {
    pub fn new(
        name: &'a str,
        tx: mpsc::Sender<AgentMessage<'a>>,
        rx: mpsc::Receiver<AgentMessage<'a>>,
        config_list: Vec<Config>,
    ) -> Self {
        let ctx = Arc::new(AgentCtx { name, tx });
        Self {
            ctx,
            rx: Some(rx),
            config_list,
            messages: HashMap::new(),
            reply_fn_list: Vec::new(),
        }
    }
}

impl<'a> Agent<'a> for Assistant<'a> {
    async fn run(&mut self) {
        let mut rx = std::mem::take(&mut self.rx).unwrap();

        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                Send { message, recipient, request_reply: _ } => {
                    println!("sent {message} from {}", recipient.name);
                }
                Receive { recipient, message, request_reply: _ } => {
                    println!(
                        "{} received from {}: {message}",
                        self.ctx.name, recipient.name
                    );
                }
                _ => {}
            }
        }
    }
}
