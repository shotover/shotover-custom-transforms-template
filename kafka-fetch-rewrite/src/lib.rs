use anyhow::Result;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use kafka_protocol::records::{
    Compression, RecordBatchDecoder, RecordBatchEncoder, RecordEncodeOptions,
};
use serde::{Deserialize, Serialize};
use shotover::frame::kafka::{KafkaFrame, ResponseBody};
use shotover::frame::{Frame, MessageType};
use shotover::message::Messages;
use shotover::transforms::{
    ChainState, DownChainProtocol, Transform, TransformBuilder, TransformConfig,
    TransformContextBuilder, TransformContextConfig, UpChainProtocol,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct KafkaFetchRewriteConfig {
    pub result: String,
}

const NAME: &str = "KafkaFetchRewrite";
#[typetag::serde(name = "KafkaFetchRewrite")]
#[async_trait(?Send)]
impl TransformConfig for KafkaFetchRewriteConfig {
    async fn get_builder(
        &self,
        _transform_context: TransformContextConfig,
    ) -> Result<Box<dyn TransformBuilder>> {
        Ok(Box::new(KafkaFetchRewriteBuilder {
            result: self.result.clone(),
        }))
    }

    fn up_chain_protocol(&self) -> UpChainProtocol {
        UpChainProtocol::MustBeOneOf(vec![MessageType::Kafka])
    }

    fn down_chain_protocol(&self) -> DownChainProtocol {
        DownChainProtocol::SameAsUpChain
    }
}

#[derive(Clone)]
pub struct KafkaFetchRewriteBuilder {
    result: String,
}

impl TransformBuilder for KafkaFetchRewriteBuilder {
    fn build(&self, _transform_context: TransformContextBuilder) -> Box<dyn Transform> {
        Box::new(KafkaFetchRewrite {
            result: self.result.clone(),
        })
    }

    fn get_name(&self) -> &'static str {
        NAME
    }
}

pub struct KafkaFetchRewrite {
    result: String,
}

#[async_trait]
impl Transform for KafkaFetchRewrite {
    fn get_name(&self) -> &'static str {
        NAME
    }

    async fn transform<'shorter, 'longer: 'shorter>(
        &mut self,
        chain_state: &'shorter mut ChainState<'longer>,
    ) -> Result<Messages> {
        let mut responses = chain_state.call_next_transform().await?;

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
                                RecordBatchDecoder::decode::<
                                    _,
                                    fn(&mut bytes::Bytes, Compression) -> Result<Bytes>,
                                >(&mut records_bytes.clone())
                            {
                                for record in &mut records {
                                    if record.value.is_some() {
                                        record.value =
                                            Some(Bytes::from(self.result.as_bytes().to_vec()));
                                    }
                                }

                                let mut new_bytes = BytesMut::new();
                                RecordBatchEncoder::encode::<
                                    _,
                                    _,
                                    fn(&mut BytesMut, &mut BytesMut, Compression) -> Result<()>,
                                >(
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
