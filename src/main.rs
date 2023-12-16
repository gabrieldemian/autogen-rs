use autogen::{
    agent::{
        assistant::AssistantAgent, Agent, AgentMessage, AgentReplyTrigger,
    },
    builder::{AssistantBuilder, Builder, UserProxyBuilder},
    config::Config,
};
use std::error::Error;
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_list = Config::from_file("config.json")?;

    let mut user = UserProxyBuilder::new("user_proxy")
        .config_list(config_list.clone())
        .build();

    let mut assistant =
        AssistantBuilder::new("assistant").config_list(config_list).build();

    // we can register a custom repply that will be triggered
    // when a specific agent sends a message.
    assistant.register_repply(
        AgentReplyTrigger(assistant.ctx.name),
        Box::new(|_agent| {
            println!("will be called when user sends a message");
        }),
    );

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
            message: "What date is today? Compare the year-to-date gain for META and TESLA.".to_owned(),
            recipient: assistant_ctx,
            request_reply: true,
        })
        .await?;

    handle.await?;

    Ok(())
}
