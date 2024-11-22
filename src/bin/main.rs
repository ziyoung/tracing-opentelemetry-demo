use std::time::Duration;

use opentelemetry::{global::{self}, KeyValue};
use opentelemetry_sdk::{runtime, trace::Tracer, Resource};
use opentelemetry_semantic_conventions::{resource::{SERVICE_NAME, SERVICE_VERSION}, SCHEMA_URL};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};
use opentelemetry::trace::TracerProvider as _;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();

    foo_example().await;

    // make sure all tracing data had been send to jaeger
    // debug info will display in terminal
    tokio::time::sleep(Duration::from_secs(5)).await;
    opentelemetry::global::shutdown_tracer_provider();
}

pub async fn foo_example() {
    let span = tracing::info_span!("foo span");
    let _enter = span.enter();
    let text = biz().await;
    log::info!("foo() return value -> {}", text);
}

async fn biz() -> String {
    let span = tracing::info_span!("biz span");
    let _enter = span.enter();
    let input = biz_inner().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    log::info!("biz done");
    format!("{},biz", input)
}

async fn biz_inner() -> String {
    let span = tracing::info_span!("biz_inner span", x=123, y=456);
    let _enter = span.enter();
    tokio::time::sleep(Duration::from_secs(1)).await;
    "biz_inner".to_string()
}

fn init_tracing_subscriber() {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env())
        .unwrap();

    let log_format = fmt::format()
        .with_target(false)
        .with_ansi(false)
        .with_source_location(true)
        .compact();
    let mut fmt_layer = fmt::layer().event_format(log_format);
    fmt_layer.set_ansi(false);
    
    let telemetry = tracing_opentelemetry::layer().with_tracer(init_tracer());
    let layered = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry);
    layered.init();
}

// following code comes from opentelemetry-otlp example https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs
fn resource() -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            // KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
        ],
        SCHEMA_URL,
    )
}

fn init_tracer() -> Tracer  {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
    .with_config(
        opentelemetry_sdk::trace::Config::default()
        .with_resource(resource()),
    )
    // .with_simple_exporter(exporter) // stuck
    .with_batch_exporter(exporter, runtime::Tokio)
    .build();

    global::set_tracer_provider(provider.clone());
    provider.tracer("readme_example")
}
