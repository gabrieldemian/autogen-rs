use autogen::{
    agent::{Agent, AgentMessage, AssistantAgent, UserProxy, Assistant},
    builder::{AssistantBuilder, Builder, UserProxyBuilder},
    config::Config,
};
use std::error::Error;
use tokio::spawn;

struct CustomAgent {
    nada: String,
}

impl<'a> Agent<'a> for CustomAgent {
    async fn run(&mut self) {}
}

impl<'a> AssistantAgent<'a> for CustomAgent {
    fn register_repply<T: Agent<'a>>(
        &mut self,
        trigger: autogen::agent::AgentReplyTrigger<'a, T>,
        function: autogen::agent::AgentReplyFn,
    ) {
        //
    }
    // async fn run(&mut self) {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_list = Config::from_file("config.json")?;

    let mut user = UserProxyBuilder::new("user_proxy")
        .config_list(config_list.clone())
        .build::<UserProxy<'_>>();

    let mut assistant = AssistantBuilder::new("assistant")
        .config_list(config_list)
        .build::<Assistant<'_>>();

    let user_ctx = user.ctx.clone();
    let assistant_ctx = assistant.ctx.clone();

    let handle = spawn(async move {
        spawn(async move {
            assistant.run().await;
        });

        user.run().await;
    });

    user_ctx
        .tx
        .send(AgentMessage::InitiateChat {
            message: "initiating chat with the first message.".to_owned(),
            recipient: assistant_ctx,
            request_reply: true,
        })
        .await?;

    handle.await?;

    Ok(())
}
