use anyhow::Result;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use kafka_protocol::records::{
    Compression, RecordBatchDecoder, RecordBatchEncoder, RecordEncodeOptions,
};
use serde::Deserialize;
use shotover::frame::kafka::{KafkaFrame, ResponseBody};
use shotover::frame::Frame;
use shotover::message::Messages;
use shotover::transforms::{Transform, TransformBuilder, TransformConfig, Transforms, Wrapper};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct KafkaFetchRewriteConfig {
    pub result: String,
}

#[typetag::deserialize(name = "KafkaFetchRewrite")]
#[async_trait(?Send)]
impl TransformConfig for KafkaFetchRewriteConfig {
    async fn get_builder(&self, _chain_name: String) -> Result<Box<dyn TransformBuilder>> {
        Ok(Box::new(KafkaFetchRewriteBuilder {
            result: self.result.clone(),
        }))
    }
}

#[derive(Clone)]
pub struct KafkaFetchRewriteBuilder {
    result: String,
}

impl TransformBuilder for KafkaFetchRewriteBuilder {
    fn build(&self) -> Transforms {
        Transforms::Custom(Box::new(KafkaFetchRewrite {
            result: self.result.clone(),
        }))
    }

    fn get_name(&self) -> &'static str {
        "KafkaFetchRewrite"
    }
}

pub struct KafkaFetchRewrite {
    result: String,
}

#[async_trait]
impl Transform for KafkaFetchRewrite {
    async fn transform<'a>(&'a mut self, message_wrapper: Wrapper<'a>) -> Result<Messages> {
        let mut responses = message_wrapper.call_next_transform().await?;

        for response in &mut responses {
            if let Some(Frame::Kafka(KafkaFrame::Response {
                body: ResponseBody::Fetch(fetch),
                ..
            })) = response.frame()
            {
                for response in &mut fetch.responses {
                    for partition in &mut response.partitions {
                        if let Some(records_bytes) = &mut partition.records {
                            if let Ok(mut records) =
                                RecordBatchDecoder::decode(&mut records_bytes.clone())
                            {
                                for record in &mut records {
                                    if record.value.is_some() {
                                        record.value =
                                            Some(Bytes::from(self.result.as_bytes().to_vec()));
                                    }
                                }

                                let mut new_bytes = BytesMut::new();
                                RecordBatchEncoder::encode(
                                    &mut new_bytes,
                                    records.iter(),
                                    &RecordEncodeOptions {
                                        version: 0, // TODO: get this from somewhere
                                        compression: Compression::None,
                                    },
                                )?;
                                *records_bytes = new_bytes.freeze();
                            }
                        }
                    }
                }
                response.invalidate_cache();
            }
        }

        Ok(responses)
    }
}
