shotover::import_transform!(redis_get_rewrite::RedisGetRewriteConfig);
shotover::import_transform!(kafka_fetch_rewrite::KafkaFetchRewriteConfig);

fn main() {
    shotover::runner::Shotover::new().run_block();
}
