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

use ledgera_pki::message::AuthenticatableMessage;

pub trait PubSubNetwork: Send + Sized + 'static {
    type Configuration;

    type IncomingQuery: std::fmt::Debug + Send + Sized + 'static;

    type CommRuntimeError: Clone + PartialEq + Eq + std::fmt::Debug + Send + Sized + 'static;

    /**
    Create a session object from a configuration.
    **/
    fn connect(
        config: Self::Configuration,
    ) -> impl std::future::Future<Output = Result<Self, Self::CommRuntimeError>> + Send;

    /**
     Publish a message of a specific kind over the network.
    The targets of the emission depend entirely of the kind of message,
    which corresponds to a topic in PubSub.
     **/
    fn publish(
        &mut self,
        topic_str: &str,
        msg: AuthenticatableMessage,
    ) -> impl std::future::Future<Output = Result<(), Self::CommRuntimeError>> + Send;

    /**
     "storage nodes" are subscribed to "VerifiedStorageRequest"
     "computer nodes" are subscribed to "ComputationRelatedCoreRequest" and "Vote"
     "logger nodes" are subscribed to "Transaction"
    etc
    Once subscribed, messages published on the corresponding topic
    will be written on the tokio::sync::mpsc channel "published_messages_sender"
     **/
    fn subscribe(
        &mut self,
        topic_str: &str,
        published_messages_sender: tokio::sync::mpsc::Sender<AuthenticatableMessage>,
    ) -> impl std::future::Future<Output = Result<(), Self::CommRuntimeError>> + Send;

    /**
     Declares an endpoint from which to receive incoming queries on a specific topic.
    These incoming queries, as well as their payloads, which must be mapped to an "AuthenticatableMessage",
    are pushed into the "incoming_queries_sender".
     **/
    fn declare_queryable(
        &mut self,
        topic_str: &str,
        incoming_queries_sender: tokio::sync::mpsc::Sender<(
            Self::IncomingQuery,
            AuthenticatableMessage,
        )>,
    ) -> impl std::future::Future<Output = Result<(), Self::CommRuntimeError>> + Send;

    /**
    Provide a means to reply to incoming queries.
    **/
    fn reply_to_incoming_query(
        incoming_query: &Self::IncomingQuery,
        response: AuthenticatableMessage,
    ) -> impl std::future::Future<Output = Result<(), Self::CommRuntimeError>> + Send;

    /**
     Sends a query on a specific topic with a "AuthenticatableMessage" as payload and
    expect responses that will be put into the "responses_sender".
     **/
    fn query_network(
        &mut self,
        topic_str: &str,
        query_message: AuthenticatableMessage,
        responses_sender: tokio::sync::mpsc::Sender<AuthenticatableMessage>,
    ) -> impl std::future::Future<Output = Result<(), Self::CommRuntimeError>> + Send;
}
