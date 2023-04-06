use crate::{docker_compose, shotover};
use redis::aio::{AsyncStream, Connection};
use redis::{Cmd, RedisConnectionInfo};
use std::pin::Pin;

pub async fn assert_ok(cmd: &mut Cmd, connection: &mut Connection) {
    assert_eq!(cmd.query_async(connection).await, Ok("OK".to_string()));
}

pub async fn assert_bytes(cmd: &mut Cmd, connection: &mut Connection, value: &[u8]) {
    assert_eq!(cmd.query_async(connection).await, Ok(value.to_vec()));
}

pub async fn redis_connection(port: u16) -> redis::aio::Connection {
    let address = "127.0.0.1";
    let stream = tokio::net::TcpStream::connect((address, port))
        .await
        .unwrap();
    let stream = Box::pin(stream) as Pin<Box<dyn AsyncStream + Send + Sync>>;

    redis::aio::Connection::new(&RedisConnectionInfo::default(), stream)
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn test_redis_get_rewrite() {
    // Setup shotover and the redis server it connects to
    let _compose = docker_compose("redis-get-rewrite-config/docker-compose.yaml");
    let shotover = shotover("redis-get-rewrite-config/topology.yaml").await;
    let mut connection = redis_connection(6379).await;

    // Verify functionality of transform
    assert_ok(
        redis::cmd("SET").arg("foo").arg("some value"),
        &mut connection,
    )
    .await;
    assert_bytes(
        redis::cmd("GET").arg("foo"),
        &mut connection,
        b"Rewritten value",
    )
    .await;
    assert_bytes(
        redis::cmd("GET").arg("bar"),
        &mut connection,
        b"Rewritten value",
    )
    .await;

    // Shutdown shotover asserting that it encountered no errors
    shotover.shutdown_and_then_consume_events(&[]).await;
}
