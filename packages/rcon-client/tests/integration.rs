use std::time::Duration;

use rcon_client::{ConnectionHandle, ConnectionRegistry, ConnectionState, RconConfig, ServerId};
use secrecy::SecretString;

mod mock_server;

use mock_server::{MockBehaviour, MockRconServer};

fn config_for(addr: std::net::SocketAddr, password: &str) -> RconConfig {
    let mut cfg = RconConfig::new(
        addr.ip().to_string(),
        addr.port(),
        SecretString::from(password),
    );
    cfg.connect_timeout = Duration::from_secs(2);
    cfg.command_timeout = Duration::from_secs(1);
    cfg
}

#[tokio::test]
async fn opens_authenticates_and_sends_command() {
    let server = MockRconServer::start(MockBehaviour::default())
        .await
        .unwrap();
    let handle = ConnectionHandle::open(config_for(server.addr, "correct-horse"));

    let settled = handle.wait_until_settled().await;
    assert_eq!(settled, ConnectionState::Open);

    let response = handle.send("listplayers").await.unwrap();
    assert_eq!(response, "pong");

    handle.close().await;
    server.stop().await;
}

#[tokio::test]
async fn auth_failure_lands_in_failed_state() {
    let server = MockRconServer::start(MockBehaviour::default())
        .await
        .unwrap();
    let handle = ConnectionHandle::open(config_for(server.addr, "wrong-password"));

    let settled = handle.wait_until_settled().await;
    assert_eq!(settled, ConnectionState::Failed);

    // Any send while Failed returns NotConnected immediately.
    let err = handle.send("listplayers").await.unwrap_err();
    assert!(matches!(err, rcon_client::RconError::NotConnected));

    handle.close().await;
    server.stop().await;
}

#[tokio::test]
async fn reconnects_after_transport_drop() {
    let behaviour = MockBehaviour {
        drop_after: Some(1),
        ..MockBehaviour::default()
    };
    let server = MockRconServer::start(behaviour).await.unwrap();
    let handle = ConnectionHandle::open(config_for(server.addr, "correct-horse"));

    assert_eq!(handle.wait_until_settled().await, ConnectionState::Open);

    // First command succeeds.
    assert_eq!(handle.send("first").await.unwrap(), "pong");

    // Second command triggers the drop. The send itself errors (the server
    // closes the socket while the client is waiting for the multi-packet
    // end-marker), state goes Reconnecting, then maintain task reconnects
    // and we settle back to Open.
    let _ = handle.send("second").await; // expected to error
    assert_eq!(handle.wait_until_settled().await, ConnectionState::Open);

    handle.close().await;
    server.stop().await;
}

#[tokio::test]
async fn close_cancels_reconnect_loop_promptly() {
    // Point the handle at a port nothing is listening on; it'll loop
    // through connect-fail + backoff. close() should still return quickly.
    let dead_addr = "127.0.0.1:1".parse().unwrap();
    let handle = ConnectionHandle::open(config_for(dead_addr, "anything"));

    // Give the maintain task a moment to enter the backoff sleep.
    tokio::time::sleep(Duration::from_millis(100)).await;

    let start = std::time::Instant::now();
    handle.close().await;
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "close took {:?}, expected <2s",
        start.elapsed()
    );
}

#[tokio::test]
async fn registry_routes_by_server_id() {
    let server_a = MockRconServer::start(MockBehaviour {
        canned_response: Some("server-a".into()),
        ..MockBehaviour::default()
    })
    .await
    .unwrap();
    let server_b = MockRconServer::start(MockBehaviour {
        canned_response: Some("server-b".into()),
        ..MockBehaviour::default()
    })
    .await
    .unwrap();

    let registry = ConnectionRegistry::new();
    let id_a = ServerId::new();
    let id_b = ServerId::new();
    registry
        .open(id_a, config_for(server_a.addr, "correct-horse"))
        .await;
    registry
        .open(id_b, config_for(server_b.addr, "correct-horse"))
        .await;

    // Wait for both to settle.
    {
        let h = registry.handle(id_a).await.unwrap();
        assert_eq!(h.wait_until_settled().await, ConnectionState::Open);
    }
    {
        let h = registry.handle(id_b).await.unwrap();
        assert_eq!(h.wait_until_settled().await, ConnectionState::Open);
    }

    assert_eq!(registry.send(id_a, "x").await.unwrap(), "server-a");
    assert_eq!(registry.send(id_b, "x").await.unwrap(), "server-b");

    let unknown = ServerId::new();
    let err = registry.send(unknown, "x").await.unwrap_err();
    assert!(matches!(err, rcon_client::RconError::UnknownServer));

    registry.close_all().await;
    server_a.stop().await;
    server_b.stop().await;
}
