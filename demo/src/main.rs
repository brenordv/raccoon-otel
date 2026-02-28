use std::time::Duration;

use raccoon_otel::{OtelOptions, Protocol};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = std::env::args().nth(1);

    let mut builder = OtelOptions::builder()
        .protocol(Protocol::HttpProtobuf)
        .resource_attributes([
            ("deployment.environment", "demo"),
            ("service.version", "1.0.0"),
        ]);

    if let Some(ref url) = endpoint {
        builder = builder.endpoint(url);
    }

    let _guard = raccoon_otel::setup_otel("raccoon-otel-demo", Some(builder.build()))?;

    tracing::info!("Demo application started");

    fetch_user("user-42").await;
    process_order("order-123", 3).await;

    tracing::info!("Demo application finished");

    Ok(())
}

#[tracing::instrument]
async fn fetch_user(user_id: &str) {
    tracing::info!(user_id, "Fetching user from database");
    simulate_work(Duration::from_millis(50)).await;
    tracing::debug!(user_id, "User fetched successfully");
}

#[tracing::instrument]
async fn process_order(order_id: &str, item_count: u32) {
    tracing::info!(order_id, item_count, "Processing order");

    for i in 1..=item_count {
        process_item(order_id, i).await;
    }

    tracing::info!(order_id, "Order processed successfully");
}

#[tracing::instrument(skip(order_id))]
async fn process_item(order_id: &str, item_number: u32) {
    tracing::debug!(order_id, item_number, "Processing item");
    simulate_work(Duration::from_millis(30)).await;

    if item_number == 2 {
        tracing::warn!(order_id, item_number, "Item required retry");
        simulate_work(Duration::from_millis(20)).await;
    }
}

async fn simulate_work(duration: Duration) {
    tokio::time::sleep(duration).await;
}
