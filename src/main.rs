use std::sync::Arc;
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
    let mut bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .on_event(|event, client| async move {
            match event {
                Event::PairingQrCode { code, .. } => {
                    println!("Scan this QR code with WhatsApp:\n{}", code);
                }
                Event::Message(msg, info) => {
                    println!("Message from {}: {:?}", info.source.sender, msg);
                }
                _ => {}
            }
        })
        .build()
        .await?;

    // Start the bot
    bot.run().await?.await?;
    Ok(())
}