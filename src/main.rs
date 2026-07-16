use std::sync::Arc;
use whatsapp_rust::TokioRuntime;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use wacore::types::events::Event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize storage backend
    let backend = Arc::new(SqliteStore::new("whatsapp.db").await?);

    // Build the bot
    let app = Bot::builder()
        .with_backend_arc(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, _client| async move {
            match &*event {
                Event::PairingQrCode(qr) => {
                    println!("Scan this QR code with WhatsApp:\n{}", qr.code);
                }
                Event::Messages(batch) => {
                    for inbound in batch.messages.iter() {
                        println!("Message from {}: {:?}", inbound.info.source.sender, inbound.message);
                    }
                }
                Event::Connected(_e) => {
                    println!("Connected to WhatsApp");
                }
                _ => {}
            }
        })
        .with_runtime(TokioRuntime)
        .build();
        let app = app.await?;
        let _g = app.client();

    // Start the bot (run now consumes the bot and drives the loop to completion)
    app.run().await;
    Ok(())
}
