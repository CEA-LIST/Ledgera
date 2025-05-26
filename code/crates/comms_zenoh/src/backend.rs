/**************************************************************************************************
 * Copyright (c) 2025 CEA (Commissariat à l'énergie atomique et aux énergies alternatives)
 *   contributors:
 *   - Erwan Mahe ( erwan.mahe@cea.fr )
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *       https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 *************************************************************************************************/

use ledgera_comms::comm_session::PubSubNetwork;
use std::borrow::Cow;
use tokio::sync::mpsc::Sender;

use crate::error::LedgeraZenohBackendError;

use ledgera_pki::message::AuthenticatableMessage;
use zenoh::Session as ZSession;

pub struct ZenohBackend {
    z_session: ZSession,
}

impl PubSubNetwork for ZenohBackend {
    type Configuration = zenoh::config::Config;
    type IncomingQuery = zenoh::query::Query;
    type CommRuntimeError = LedgeraZenohBackendError;

    async fn connect(config: Self::Configuration) -> Result<Self, Self::CommRuntimeError> {
        let z_session = zenoh::open(config).await.unwrap();
        Ok(Self { z_session })
    }

    async fn publish(
        &mut self,
        topic_str: &str,
        msg: AuthenticatableMessage,
    ) -> Result<(), Self::CommRuntimeError> {
        match self
            .z_session
            .put(topic_str, &bincode::serialize(&msg).unwrap())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(LedgeraZenohBackendError::ZenohError(format!("{:?}", e))),
        }
    }

    async fn subscribe(
        &mut self,
        topic_str: &str,
        published_messages_sender: Sender<AuthenticatableMessage>,
    ) -> Result<(), Self::CommRuntimeError> {
        let subscription_result = self.z_session.declare_subscriber(topic_str).await;
        match subscription_result {
            Ok(subscriber) => {
                tokio::spawn(async move {
                    loop {
                        let received_zresult = subscriber.recv_async().await;
                        match received_zresult {
                            Err(e) => {
                                log::warn!(
                                    "Zenoh backend : subscribed topic runtime error {:?}",
                                    e
                                );
                            }
                            Ok(z_sample) => {
                                let sample_key_expr_as_str = z_sample.key_expr().as_str();
                                log::debug!(
                                    "Zenoh backend : received message on topic {:}",
                                    sample_key_expr_as_str
                                );
                                let payload_as_bytes: Cow<[u8]> = z_sample.payload().to_bytes();
                                match bincode::deserialize::<AuthenticatableMessage>(
                                    &payload_as_bytes,
                                ) {
                                    Err(e) => {
                                        log::warn!(
                                                "Zenoh backend : received message on topic {:} but could not deserialize content with error {:?}",
                                                sample_key_expr_as_str,
                                                e
                                            );
                                    }
                                    Ok(msg) => {
                                        if let Err(e) = published_messages_sender.send(msg).await {
                                            log::warn!(
                                                    "Zenoh backend : could not retransmit deserialized message to Ledgera with error : {:?}",
                                                    e
                                                );
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
                Ok(())
            }
            Err(e) => Err(LedgeraZenohBackendError::ZenohError(format!("{:?}", e))),
        }
    }

    async fn declare_queryable(
        &mut self,
        topic_str: &str,
        incoming_queries_sender: Sender<(Self::IncomingQuery, AuthenticatableMessage)>,
    ) -> Result<(), Self::CommRuntimeError> {
        let queryable_result = self.z_session.declare_queryable(topic_str).await;
        match queryable_result {
            Ok(queryable) => {
                tokio::spawn(async move {
                    loop {
                        let received_zresult = queryable.recv_async().await;
                        match received_zresult {
                            Err(e) => {
                                log::warn!("Zenoh backend : query reception runtime error {:?}", e);
                            }
                            Ok(query) => {
                                let query_key_expr_as_str = query.key_expr().as_str();
                                match query.payload() {
                                    None => {
                                        log::warn!(
                                            "Zenoh backend : received query on topic {:} without any payload",
                                            query_key_expr_as_str,
                                        );
                                    }
                                    Some(query_payload) => {
                                        let payload_as_bytes: Cow<[u8]> = query_payload.to_bytes();
                                        match bincode::deserialize::<AuthenticatableMessage>(
                                            &payload_as_bytes,
                                        ) {
                                            Err(e) => {
                                                log::warn!(
                                                    "Zenoh backend : received query on topic {:} but could not deserialize content with error {:?}",
                                                    query_key_expr_as_str,
                                                    e
                                                );
                                            }
                                            Ok(msg) => {
                                                if let Err(e) =
                                                    incoming_queries_sender.send((query, msg)).await
                                                {
                                                    log::warn!(
                                                        "Zenoh backend : could not retransmit deserialized message to Ledgera query responses sender with error : {:?}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
                Ok(())
            }
            Err(e) => Err(LedgeraZenohBackendError::ZenohError(format!("{:?}", e))),
        }
    }

    async fn reply_to_incoming_query(
        incoming_query: &Self::IncomingQuery,
        response: AuthenticatableMessage,
    ) -> Result<(), Self::CommRuntimeError> {
        match incoming_query
            .reply(
                incoming_query.key_expr(),
                &bincode::serialize(&response).unwrap(),
            )
            .await
        {
            Err(e) => Err(LedgeraZenohBackendError::ZenohError(format!("{:?}", e))),
            Ok(_) => Ok(()),
        }
    }

    async fn query_network(
        &mut self,
        topic_str: &str,
        query_message: AuthenticatableMessage,
        responses_sender: Sender<AuthenticatableMessage>,
    ) -> Result<(), Self::CommRuntimeError> {
        let zenoh_query = self
            .z_session
            .get(topic_str)
            .payload(bincode::serialize(&query_message).unwrap());
        let replies = zenoh_query.await.unwrap();
        tokio::spawn(async move {
            let mut num_replies = 0_u32;
            'waiting_for_replies: loop {
                let received_zresult = replies.recv_async().await;
                num_replies += 1;
                match received_zresult {
                    Err(e) => {
                        if num_replies == 0 {
                            log::warn!("Zenoh backend : query reply fatal runtime error : {:?}", e);
                        } else {
                            log::debug!("Zenoh backend : no more replies :  {:?}", e);
                        }
                        break 'waiting_for_replies;
                    }
                    Ok(reply) => {
                        match reply.into_result() {
                            Err(e) => {
                                log::warn!(
                                    "Zenoh backend : query reply benign runtime error {:?}",
                                    e
                                );
                            }
                            Ok(z_sample) => {
                                // ***
                                let sample_key_expr_as_str = z_sample.key_expr().as_str();
                                log::debug!(
                                    "Zenoh backend : received query reply on topic {:}",
                                    sample_key_expr_as_str
                                );
                                let payload_as_bytes: Cow<[u8]> = z_sample.payload().to_bytes();
                                match bincode::deserialize::<AuthenticatableMessage>(
                                    &payload_as_bytes,
                                ) {
                                    Err(e) => {
                                        log::warn!(
                                            "Zenoh backend : received query reply on topic {:} but could not deserialize content with error {:?}",
                                            sample_key_expr_as_str,
                                            e
                                        );
                                    }
                                    Ok(msg) => {
                                        if let Err(e) = responses_sender.send(msg).await {
                                            log::warn!(
                                                "Zenoh backend : could not retransmit deserialized message to Ledgera query responses sender with error : {:?}",
                                                e
                                            );
                                        }
                                    }
                                }
                                // ***
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
