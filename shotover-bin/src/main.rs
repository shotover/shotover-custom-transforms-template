shotover::import_transform!(valkey_get_rewrite::ValkeyGetRewriteConfig);
shotover::import_transform!(kafka_fetch_rewrite::KafkaFetchRewriteConfig);

fn main() {
    shotover::runner::Shotover::new().run_block();
}
