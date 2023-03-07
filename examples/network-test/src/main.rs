use optimizely::{ClientBuilder, event::BatchedEventDispatcher};
use std::error::Error;
use env_logger::Target;
use log::LevelFilter;

const SDK_KEY: &str = "KVpGWnzPGKvvQ8yeEWmJZ";

fn main() -> Result<(), Box<dyn Error>> {
    // Always set log level to debug
    env_logger::builder()
        .target(Target::Stdout)
        .filter_module("optimizely", LevelFilter::Debug)
        .init();

    // Initiate client using SDK key and batched event dispatcher
    let client = ClientBuilder::new()
        .with_sdk_key(SDK_KEY)?
        .with_event_dispatcher(BatchedEventDispatcher::new())
        .build()?;
    
    let flag_key = "buy_button";
    let decide_options = vec![];

    for i in 0..20 {
        let user_id = format!("user{}", i);
        let user_context = client.create_user_context(&user_id);
        let _decision = user_context.decide(flag_key, &decide_options);
    }

    Ok(())
}