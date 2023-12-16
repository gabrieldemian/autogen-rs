# initialization

The first step is to load the configuration. This can be done be reading a json file, or by manually initialing the struct. This configuration will be passed to all the agents.

```rs
let config: Vec<Config> = Config::from_file("config.json").unwrap();
```

# Agents

We initialize the agents using the builder pattern. This pattern is great when you have many optional values or you want to hide initialization logic, which is the case here.

```rs
// ...
let mut assistant = AssistantBuilder::new("assistant").config_list(config.clone()).build();

let mut user_proxy = UserProxyBuilder::new("user_proxy").config_list(config).build();
// let result = user_proxy.chat(&assistant).clear_history(true).message("Plot a chart of NVDA and TESLA stock price change YTD.").initialize();
```

## Advantages
- Implementation details hidden on the `build` function.
- Succint API.

# Agents Communication

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

## Advantages
- Async communication (possibility of safe multi-threaded) with the option to opt-in to sync. The AutoGen in python has duplicate functions for async/sync.
- Idiomatic Rust code with `mpsc` channels.
- Communication can happen with just the `ctx` of the agent, no need to pass the entire struct as an argument.
- Strongly typed messages.

The `run` function starts the event-loop. It will listen for messages of the type `AgentMessage`.

## Custom Agents
Custom agents can be done by creating a custom struct that implements the `Agent` and `AssistantAgent` trait. Aditionaly, it is possible to reuse the builder trait `Builder`.

Custom replies can be done by calling `register_repply`.

Like the original AutoGen, custom repplies can be triggered by a specific agent trait or name.
