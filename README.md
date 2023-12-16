# initialization

The first step is to load the configuration. This can be done be reading a json file, or by manually initialing the struct. This configuration will be passed to all the agents.

```rs
let config_list: Vec<Config> = Config::from_file("config.json").unwrap();
// or
let config_list: Vec<Config> = serde_json::from_str(
    r#"[ { "model": "gpt-4" } ]"#,
)
.unwrap();
```

# Agents
## Initializing
We initialize the agents using the builder pattern. This pattern is great when you have many optional values or you want to hide initialization logic, which is the case here.

```rs
// ...
let mut assistant = AssistantBuilder::new("assistant").config_list(config.clone()).build();

let mut user_proxy = UserProxyBuilder::new("user_proxy").config_list(config).build();
```

### Advantages
- Agent initialization details hidden on the `build` function.
- Flexible custom agents, with their own data structures, and custom functions to handle messages.

## Agents Communication

Agents communicate with each other by using Rust built-in mpsc channel. Actually, the code uses the tokio version of mpsc, which is async, but there is the option to use sync as well.

```rs
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
```

The messages can be fired by using the `tx` from the agent context, it is also possible to not spawn the event loop and call the functions directly on the struct.

### Advantages
- Async communication (possibility of safe multi-threaded) with the option to opt-in to sync. The AutoGen in python has duplicate functions for async/sync.
- Idiomatic Rust code with `mpsc` channels.
- Communication can happen with just the `ctx` of the agent, no need to pass the entire struct as an argument.
- Strongly typed messages.

The `run` function starts the event-loop. It will listen for messages of the type `AgentMessage`.

## Custom Agents
Users can implement custom agents with their own data structures, custom logic/replies to built-in messages, and more.

Like the original AutoGen, custom repplies can be triggered by a specific agent trait or name.

Custom agents can be done by creating a custom struct that implements the `Agent` and `AssistantAgent` trait. Aditionaly, it is possible to reuse the builder trait `Builder` to build this custom agent.

The `AssistantAgent` handles logic that is specific to assistants, in this case, it requires the function `request_reply` to be implemented.

The `Agent` handles logic for all agents, the `run` function and some constants strings, like model, description, system message, etc.

Custom replies can be done by calling `register_repply`.

### Advantages
- Flexible custom agents, with their own data structures, and custom functions to handle messages.

### Disadvantages
- More verbosity to implement custom agents.

```rs
struct CustomAgent {
    name: String,
}

impl<'a> Agent<'a> for CustomAgent {
    async fn run(&mut self) {
        // handle messages here
    }
}

impl<'a> AssistantAgent<'a> for CustomAgent {
    fn register_repply<T: Agent<'a>>(
        &mut self,
        trigger: autogen::agent::AgentReplyTrigger<'a, T>,
        function: autogen::agent::AgentReplyFn,
    ) {
        // implement logic to register custom replies.
    }
}
```
