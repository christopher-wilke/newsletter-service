use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, prelude::__tracing_subscriber_SubscriberExt};

/// Compose multiple layers into a `tracing`'s subscriber.
/// 
/// # Implementation Notes
/// 
/// We are using `impl Subscriber` as return type to avoid having
/// to spell out the actualy type of the returned subscriber, which is need quite complex.
/// 
/// We need to explicitly call out that the returned subscriber is `Send` and `Sync` to make it possible
/// to pass it it to `init_subscriber` later on.
pub fn get_subscriber(
    name: String,
    env_filter: String
) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(
        name.into(),
        std::io::stdout
    );

     Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Registers a subscriber as global default to process span data
/// 
/// It should only be called once!
pub fn init_subscriber(
    subscriber: impl Subscriber + Send + Sync
) {
    LogTracer::init().expect("failed to set logger");
    set_global_default(subscriber).expect("Fauled to set subscriber");
}