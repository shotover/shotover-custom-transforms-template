mod kafka;
mod redis;

use docker_compose_runner::{DockerCompose, Image};
use std::time::Duration;
use tokio_bin_process::event::Level;
use tokio_bin_process::event_matcher::EventMatcher;
use tokio_bin_process::{bin_path, BinProcess};

fn docker_compose(yaml_path: &str) -> DockerCompose {
    DockerCompose::new(&IMAGE_WAITERS, |_| {}, yaml_path)
}

pub static IMAGE_WAITERS: [Image; 2] = [
    Image {
        name: "library/redis:5.0.9",
        log_regex_to_wait_for: r"Ready to accept connections",
        timeout: Duration::from_secs(120),
    },
    Image {
        name: "bitnami/kafka:3.4.0-debian-11-r22",
        log_regex_to_wait_for: r"Kafka Server started",
        timeout: Duration::from_secs(120),
    },
];

async fn shotover(topology_path: &str) -> BinProcess {
    let mut shotover = BinProcess::start_binary(
        bin_path!("shotover-bin"),
        "shotover",
        &["-t", topology_path, "--log-format", "json"],
    )
    .await;

    tokio::time::timeout(
        Duration::from_secs(30),
        shotover.wait_for(
            &EventMatcher::new()
                .with_level(Level::Info)
                .with_message("Shotover is now accepting inbound connections"),
            &[],
        ),
    )
    .await
    .unwrap();
    shotover
}
