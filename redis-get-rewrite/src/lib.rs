use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shotover::frame::{Frame, MessageType, RedisFrame};
use shotover::message::Messages;
use shotover::transforms::{
    ChainState, DownChainProtocol, Transform, TransformBuilder, TransformConfig,
    TransformContextBuilder, TransformContextConfig, UpChainProtocol,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RedisGetRewriteConfig {
    pub result: String,
}

const NAME: &str = "RedisGetRewrite";
#[typetag::serde(name = "RedisGetRewrite")]
#[async_trait(?Send)]
impl TransformConfig for RedisGetRewriteConfig {
    async fn get_builder(
        &self,
        _transform_context: TransformContextConfig,
    ) -> Result<Box<dyn TransformBuilder>> {
        Ok(Box::new(RedisGetRewriteBuilder {
            result: self.result.clone(),
        }))
    }

    fn up_chain_protocol(&self) -> UpChainProtocol {
        UpChainProtocol::MustBeOneOf(vec![MessageType::Redis])
    }

    fn down_chain_protocol(&self) -> DownChainProtocol {
        DownChainProtocol::SameAsUpChain
    }
}

#[derive(Clone)]
pub struct RedisGetRewriteBuilder {
    result: String,
}

impl TransformBuilder for RedisGetRewriteBuilder {
    fn build(&self, _transform_context: TransformContextBuilder) -> Box<dyn Transform> {
        Box::new(RedisGetRewrite {
            result: self.result.clone(),
        })
    }

    fn get_name(&self) -> &'static str {
        NAME
    }
}

pub struct RedisGetRewrite {
    result: String,
}

#[async_trait]
impl Transform for RedisGetRewrite {
    fn get_name(&self) -> &'static str {
        NAME
    }

    async fn transform<'shorter, 'longer: 'shorter>(
        &mut self,
        chain_state: &'shorter mut ChainState<'longer>,
    ) -> Result<Messages> {
        let mut get_indices = vec![];
        for (i, message) in chain_state.requests.iter_mut().enumerate() {
            if let Some(frame) = message.frame() {
                if is_get(frame) {
                    get_indices.push(i);
                }
            }
        }
        let mut responses = chain_state.call_next_transform().await?;

        for i in get_indices {
            if let Some(frame) = responses[i].frame() {
                rewrite_get(frame, &self.result);
                responses[i].invalidate_cache();
            }
        }

        Ok(responses)
    }
}

fn is_get(frame: &Frame) -> bool {
    if let Frame::Redis(RedisFrame::Array(array)) = frame {
        if let Some(RedisFrame::BulkString(first)) = array.first() {
            first.eq_ignore_ascii_case(b"GET")
        } else {
            false
        }
    } else {
        false
    }
}

fn rewrite_get(frame: &mut Frame, result: &str) {
    tracing::info!("Replaced {frame:?} with BulkString(\"{result}\")");
    *frame = Frame::Redis(RedisFrame::BulkString(result.to_owned().into()));
}
