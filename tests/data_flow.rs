use system_monitor::system::SystemInfoManager;
use system_monitor::app::AppMessage;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_background_collector_sends_updates() {
    // 1. Create the system manager
    let system_manager = SystemInfoManager::new().expect("Failed to create SystemInfoManager");

    // 2. Create a channel
    let (tx, mut rx) = mpsc::unbounded_channel();

    // 3. Start a background task similar to the one in app.rs
    tokio::spawn(async move {
        match system_manager.get_snapshot().await {
            Ok(snapshot) => {
                let _ = tx.send(AppMessage::SystemUpdate(snapshot));
            },
            Err(e) => {
                let _ = tx.send(AppMessage::Error(format!("Data collection failed: {}", e)));
            }
        }
    });

    // 4. Wait for a message to arrive
    let received_message = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;

    // 5. Assert that we received a SystemUpdate message
    assert!(received_message.is_ok(), "Test timed out waiting for a message.");
    let message = received_message.unwrap();
    assert!(message.is_some());

    match message.unwrap() {
        AppMessage::SystemUpdate(_) => {
            // Success!
        },
        AppMessage::Error(e) => {
            panic!("Received an error message: {}", e);
        },
        _ => {
            panic!("Received an unexpected message type.");
        }
    }
}