use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;

use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tracing::instrument(level = "info", name = "i_just_did_a_thing")]
async fn do_a_thing() -> &'static str {
    info!("I'm doing a thing!");

    "We did a thing!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT")?.parse::<u16>()?;
    setup_otel();

    info!("Hello world!");

    let router = Router::new().route("/", get(do_a_thing));

    let addr = format!("[::]:{port}");
    let tcp_listener: TcpListener = TcpListener::bind(addr).await.unwrap();

    axum::serve(tcp_listener, router).await.unwrap();

    Ok(())
}

fn setup_otel() {
    let addr = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap();

    let ctx = reqwest::Client::new();

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(addr)
                .with_http_client(ctx),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracer = provider.tracer("my_application");

    // log level filtering here
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // fmt layer - printing out logs
    let fmt_layer = fmt::layer().compact();

    // turn our OTLP pipeline into a tracing layer
    // let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // initialise our subscriber
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(OpenTelemetryLayer::new(tracer))
        .init();
}
