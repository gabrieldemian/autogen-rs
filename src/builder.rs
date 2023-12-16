use std::sync::Arc;

use hashbrown::HashMap;
use tokio::sync::mpsc;

use crate::{
    agent::{Agent, AgentCtx, AgentMessage, Assistant, UserProxy},
    config::Config,
};

/// Trait for building agents.
pub trait Builder<'a> {
    type BuildType: Agent<'a>;

    fn build(self) -> Self::BuildType;
}

impl<'a> Builder<'a> for UserProxyBuilder<'a> {
    type BuildType = UserProxy<'a>;

    fn build(self) -> Self::BuildType {
        let (tx, rx) = mpsc::channel::<AgentMessage>(100);
        let ctx = Arc::new(AgentCtx { tx, name: self.name });

        UserProxy {
            ctx,
            rx: Some(rx),
            config_list: self.config_list,
            messages: HashMap::new(),
        }
    }
}

impl<'a> Builder<'a> for AssistantBuilder<'a> {
    type BuildType = Assistant<'a>;

    fn build(self) -> Self::BuildType {
        let (tx, rx) = mpsc::channel::<AgentMessage>(100);
        let ctx = Arc::new(AgentCtx { tx, name: self.name });

        Assistant {
            ctx,
            rx: Some(rx),
            config_list: self.config_list,
            messages: HashMap::new(),
            reply_fn_list: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserProxyBuilder<'a> {
    pub name: &'a str,
    pub config_list: Vec<Config>,
}

#[derive(Debug, Default, Clone)]
pub struct AssistantBuilder<'a> {
    pub name: &'a str,
    pub config_list: Vec<Config>,
}

impl<'a> AssistantBuilder<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, ..Default::default() }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn config_list(mut self, config_list: Vec<Config>) -> Self {
        self.config_list = config_list;
        self
    }
}

impl<'a> UserProxyBuilder<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, ..Default::default() }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn config_list(mut self, config_list: Vec<Config>) -> Self {
        self.config_list = config_list;
        self
    }
}
