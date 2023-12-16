//! Assistant agent that uses LLM to finish tasks.
use std::{error::Error, sync::Arc};

use hashbrown::HashMap;
use tokio::sync::mpsc;

use crate::config::Config;

use super::{Agent, AgentCtx, AgentMessage, AgentReplyTrigger};

pub struct Assistant<'a> {
    pub ctx: Arc<AgentCtx<'a>>,
    pub config_list: Vec<Config>,
    pub messages: HashMap<AgentReplyTrigger<'a>, Vec<&'a str>>,
    pub rx: Option<mpsc::Receiver<AgentMessage<'a>>>,
    pub reply_fn_list: Vec<Box<dyn FnMut(&mut Self) + Send>>,
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

// Blanked impl for Assistant, anyone who implements Assistant
// will automatically implement AssistantAgent.
impl<'a> AssistantAgent<'a> for Assistant<'a> {
    fn register_repply(
        &mut self,
        _trigger: AgentReplyTrigger<'a>,
        function: Box<(dyn FnMut(&mut Self) + std::marker::Send + 'static)>,
    ) {
        self.reply_fn_list.push(function);
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
    async fn generate_reply(&self) -> Option<String> {
        Some("fake message".to_owned())
    }
    /// Sends a message to another agent.
    async fn send(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) {
    }
    async fn receive(
        &mut self,
        recipient: Arc<AgentCtx<'a>>,
        message: String,
        request_reply: bool,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// Trait for creating custom agents derived from `Assistant`.
pub trait AssistantAgent<'a> {
    const DEFAULT_DESCRIPTION: &'a str = "A helpful and general-purpose AI assistant that has strong language skills, Python skills, and Linux command line skills.";
    const DEFAULT_SYSTEM_MESSAGE: &'a str = r#"You are a helpful AI assistant.
Solve tasks using your coding and language skills.
In the following cases, suggest Rust code (in a Rust coding block) or shell script (in a sh coding block) for the user to execute.
    1. When you need to collect info, use the code to output the info you need, for example, browse or search the web, download/read a file, print the content of a webpage or a file, get the current date/time, check the operating system. After sufficient info is printed and the task is ready to be solved based on your language skill, you can solve the task by yourself.
    2. When you need to perform some task with code, use the code to perform the task and output the result. Finish the task smartly.
Solve the task step by step if you need to. If a plan is not provided, explain your plan first. Be clear which step uses code, and which step uses your language skill.
When using code, you must indicate the script type in the code block. The user cannot provide any other feedback or perform any other action beyond executing the code you suggest. The user can't modify your code. So do not suggest incomplete code which requires users to modify. Don't use a code block if it's not intended to be executed by the user.
If you want the user to save the code in a file before executing it, put # filename: <filename> inside the code block as the first line. Don't include multiple code blocks in one response. Do not ask users to copy and paste the result. Instead, use 'print' function for the output when relevant. Check the execution result returned by the user.
If the result indicates there is an error, fix the error and output the code again. Suggest the full code instead of partial code or code changes. If the error can't be fixed or if the task is not solved even after the code is executed successfully, analyze the problem, revisit your assumption, collect additional info you need, and think of a different approach to try.
When you find an answer, verify the answer carefully. Include verifiable evidence in your response if possible.
Reply "TERMINATE" in the end when everything is done."#;

    fn register_repply(
        &mut self,
        trigger: AgentReplyTrigger<'a>,
        // function: AgentReplyFn,
        function: Box<(dyn FnMut(&mut Self) + std::marker::Send + 'static)>,
    );
}
