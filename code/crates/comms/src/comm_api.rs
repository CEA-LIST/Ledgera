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

use ledgera_pki::manager::{
    KnownParticipantsMap, PublicKeyInfrastructure, SerdeSerializable64BitsSignature,
};
use ledgera_pki::message::{AuthenticatableMessage, SignatureEntry};
use ledgera_types::traits::LedgeraPublishableMessage;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::comm_session::PubSubNetwork;
use crate::error::{LedgeraCommunicationError, LedgeraCommunicationErrorContext};

pub struct LedgeraInternalCommunicationParameters<PKI: PublicKeyInfrastructure> {
    pub byzantine_threshold: u32,
    pub signing_key: Arc<PKI::SigningKey>,
    pub known_participants: Arc<KnownParticipantsMap<PKI::VerifyingKey>>,
}

impl<PKI: PublicKeyInfrastructure> LedgeraInternalCommunicationParameters<PKI> {
    pub fn new(
        byzantine_threshold: u32,
        signing_key: Arc<PKI::SigningKey>,
        known_participants: Arc<KnownParticipantsMap<PKI::VerifyingKey>>,
    ) -> Self {
        Self {
            byzantine_threshold,
            signing_key,
            known_participants,
        }
    }
}

pub struct LedgeraInternalCommunicationInterface<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork>
{
    phantom: PhantomData<PKI>,
    sess: Sess,
    /// Handles for the long-lived subscription/queryable tasks spawned at setup time,
    /// paired with a human-readable description of the task.
    /// Kept alive so that a silent panic in one of these loops is observable.
    /// TODO: extend to support automatic respawning — requires storing the topic string
    /// and output sender so that sess.subscribe/declare_queryable can be re-called on failure.
    task_handles: Vec<(String, tokio::task::JoinHandle<()>)>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork>
    LedgeraInternalCommunicationInterface<PKI, Sess>
{
    pub async fn from_config(
        config: Sess::Configuration,
    ) -> Result<Self, LedgeraCommunicationError<Sess::CommRuntimeError>> {
        match Sess::connect(config).await {
            Ok(sess) => Ok(Self {
                sess,
                phantom: PhantomData {},
                task_handles: Vec::new(),
            }),
            Err(e) => Err(LedgeraCommunicationError::InContext(
                LedgeraCommunicationErrorContext::AtStartup,
                Box::new(LedgeraCommunicationError::SessionError(e)),
            )),
        }
    }

    pub async fn subscribe_to_topic_and_deserialize_as<MsgType: LedgeraPublishableMessage>(
        &mut self,
        params: &LedgeraInternalCommunicationParameters<PKI>,
        topic_str: &str,
        received_messages_sender: tokio::sync::mpsc::Sender<(MsgType, SignatureEntry)>,
    ) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(128);
        match self.sess.subscribe(topic_str, sender).await {
            Ok(_) => {
                let handle =
                    tool_function_authenticate_and_deserialize_received_messages::<PKI, MsgType>(
                        params.known_participants.clone(),
                        receiver,
                        received_messages_sender,
                        topic_str,
                    );
                let description = format!(
                    "subscription on topic '{}' for {}",
                    topic_str,
                    MsgType::get_msg_type()
                );
                self.task_handles.push((description, handle));
                Ok(())
            }
            Err(e) => Err(LedgeraCommunicationError::SessionError(e)),
        }
    }

    pub async fn serialize_and_publish_on_topic<MsgType: LedgeraPublishableMessage>(
        &mut self,
        params: &LedgeraInternalCommunicationParameters<PKI>,
        topic_str: &str,
        msg_payload: &MsgType,
    ) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        match AuthenticatableMessage::create::<MsgType, PKI>(msg_payload, &params.signing_key) {
            Ok(msg) => match self.sess.publish(topic_str, msg).await {
                Ok(_) => Ok(()),
                Err(e) => Err(LedgeraCommunicationError::SessionError(e)),
            },
            Err(e) => Err(LedgeraCommunicationError::PkiError(e)),
        }
    }

    pub async fn serialize_and_publish_on_topic_returning_signature<
        MsgType: LedgeraPublishableMessage,
    >(
        &mut self,
        params: &LedgeraInternalCommunicationParameters<PKI>,
        topic_str: &str,
        msg_payload: &MsgType,
    ) -> Result<SerdeSerializable64BitsSignature, LedgeraCommunicationError<Sess::CommRuntimeError>>
    {
        match AuthenticatableMessage::create::<MsgType, PKI>(msg_payload, &params.signing_key) {
            Ok(msg) => {
                let signature = msg.signature_entry.serializable_signature.clone();
                match self.sess.publish(topic_str, msg).await {
                    Ok(_) => Ok(signature),
                    Err(e) => Err(LedgeraCommunicationError::SessionError(e)),
                }
            }
            Err(e) => Err(LedgeraCommunicationError::PkiError(e)),
        }
    }

    pub async fn declare_queryable<
        QueryType: LedgeraPublishableMessage,
        ResponseType: LedgeraPublishableMessage,
    >(
        &mut self,
        params: &LedgeraInternalCommunicationParameters<PKI>,
        topic_str: &str,
        incoming_queries_sender: tokio::sync::mpsc::Sender<(Sess::IncomingQuery, QueryType)>,
    ) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        let (raw_sender, raw_receiver) = tokio::sync::mpsc::channel(128);
        match self.sess.declare_queryable(topic_str, raw_sender).await {
            Ok(_) => {
                let handle = tool_function_authenticate_and_deserialize_received_queries::<
                    PKI,
                    Sess,
                    QueryType,
                >(
                    params.known_participants.clone(),
                    raw_receiver,
                    incoming_queries_sender,
                    topic_str,
                    QueryType::get_msg_type(),
                );
                let description = format!(
                    "queryable on topic '{}' for {}",
                    topic_str,
                    QueryType::get_msg_type()
                );
                self.task_handles.push((description, handle));
                Ok(())
            }
            Err(e) => Err(LedgeraCommunicationError::SessionError(e)),
        }
    }

    /// Checks whether any long-lived subscription or queryable task has unexpectedly
    /// terminated and logs an error for each one that has. Intended to be called
    /// periodically from the node's main loop.
    pub fn check_subscription_tasks_health(&self) {
        for (description, handle) in &self.task_handles {
            if handle.is_finished() {
                log::error!(
                    "long-lived task '{}' has unexpectedly terminated — \
                     this node may no longer be receiving messages on the affected channel",
                    description
                );
                // TODO: respawn the task by re-calling sess.subscribe/declare_queryable
                // with the stored topic string and output sender. Requires extending
                // the stored tuple to carry those parameters.
            }
        }
    }

    pub async fn query_network<
        QueryType: LedgeraPublishableMessage,
        ResponseType: LedgeraPublishableMessage,
    >(
        &mut self,
        params: &LedgeraInternalCommunicationParameters<PKI>,
        topic_str: &str,
        query_payload: &QueryType,
        responses_sender: tokio::sync::mpsc::Sender<(ResponseType, SignatureEntry)>,
    ) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        match AuthenticatableMessage::create::<QueryType, PKI>(query_payload, &params.signing_key) {
            Ok(query_message) => {
                let (raw_sender, raw_receiver) = tokio::sync::mpsc::channel(128);
                match self
                    .sess
                    .query_network(topic_str, query_message, raw_sender)
                    .await
                {
                    Ok(_) => {
                        tool_function_authenticate_and_deserialize_received_messages::<
                            PKI,
                            ResponseType,
                        >(
                            params.known_participants.clone(),
                            raw_receiver,
                            responses_sender,
                            topic_str,
                        );
                        Ok(())
                    }
                    Err(e) => Err(LedgeraCommunicationError::SessionError(e)),
                }
            }
            Err(e) => Err(LedgeraCommunicationError::PkiError(e)),
        }
    }
}

fn tool_function_authenticate_and_deserialize_received_messages<
    PKI: PublicKeyInfrastructure,
    MsgPayloadType: LedgeraPublishableMessage,
>(
    known_participants: Arc<KnownParticipantsMap<PKI::VerifyingKey>>,
    mut receiver: tokio::sync::mpsc::Receiver<AuthenticatableMessage>,
    sender: tokio::sync::mpsc::Sender<(MsgPayloadType, SignatureEntry)>,
    channel_name: &str,
) -> tokio::task::JoinHandle<()> {
    let channel_name = channel_name.to_string();
    tokio::spawn(async move {
        let known_participants = known_participants;
        while let Some(msg) = receiver.recv().await {
            match msg.authenticate::<PKI>(&known_participants) {
                Ok(_) => match bincode::deserialize::<MsgPayloadType>(&msg.serialized_payload) {
                    Ok(payload) => match sender.send((payload, msg.signature_entry)).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!(
                                "error when unpacking {} received on the {} channel : {:?}",
                                MsgPayloadType::get_msg_type(),
                                channel_name,
                                e
                            )
                        }
                    },
                    Err(e) => {
                        log::warn!(
                            "could not deserialize as {} what was received from public key {:?} on the {} channel with error {:?}",
                            MsgPayloadType::get_msg_type(),
                            msg.signature_entry.serialized_signing_public_key,
                            channel_name,
                            e
                        );
                    }
                },
                Err(e) => {
                    log::warn!(
                        "received unauthentic {} on {} channel from public key : {:?} with error {:?}",
                        MsgPayloadType::get_msg_type(),
                        channel_name,
                        msg.signature_entry.serialized_signing_public_key,
                        e
                    );
                }
            }
        }
    })
}

fn tool_function_authenticate_and_deserialize_received_queries<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    MsgPayloadType: LedgeraPublishableMessage,
>(
    known_participants: Arc<KnownParticipantsMap<PKI::VerifyingKey>>,
    mut receiver: tokio::sync::mpsc::Receiver<(Sess::IncomingQuery, AuthenticatableMessage)>,
    sender: tokio::sync::mpsc::Sender<(Sess::IncomingQuery, MsgPayloadType)>,
    channel_name: &str,
    snd_type_name: &'static str,
) -> tokio::task::JoinHandle<()> {
    let channel_name = channel_name.to_string();
    tokio::spawn(async move {
        let known_participants = known_participants;
        while let Some((query, msg)) = receiver.recv().await {
            match msg.authenticate::<PKI>(&known_participants) {
                Ok(_) => match bincode::deserialize::<MsgPayloadType>(&msg.serialized_payload) {
                    Ok(payload) => match sender.send((query, payload)).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!(
                                        "error when unpacking and retransmitting query message received on the {} channel : {:?}",
                                        channel_name,
                                        e
                                    )
                        }
                    },
                    Err(e) => {
                        log::warn!(
                                "could not deserialize as {} the query message received from public key {:?} on the {} channel with error {:?}",
                                snd_type_name,
                                msg.signature_entry.serialized_signing_public_key,
                                channel_name,
                                e
                            );
                    }
                },
                Err(e) => {
                    log::warn!(
                        "received unauthentic query message on {} channel from public key : {:?} with error {:?}",
                        channel_name,
                        msg.signature_entry.serialized_signing_public_key,
                        e
                    );
                }
            }
        }
    })
}
