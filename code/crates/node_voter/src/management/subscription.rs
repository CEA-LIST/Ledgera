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

use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
    error::LedgeraCommunicationError,
};
use ledgera_core_logic::{roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics};
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::messages::deliver::LedgeraTransactionDeliveryNotification;
use ledgera_types::requests::rfun::LedgeraRequestFunctionInstanceProposal;
use ledgera_types::requests::rin::LedgeraRequestInputProposal;
use ledgera_types::votes::{
    vfun::LedgeraVoteFunctionInstanceDeclaration, vins::LedgeraVoteIns,
    vout::LedgeraVoteFunctionOutput, vsto::LedgeraVoteStored,
};
use std::sync::Arc;

pub(crate) struct SendersForVoterNodeSubscriptions<LAT: LedgeraApplicationTemplate> {
    pub rfun_sender:
        tokio::sync::mpsc::Sender<(LedgeraRequestFunctionInstanceProposal<LAT>, SignatureEntry)>,
    pub rin_sender: tokio::sync::mpsc::Sender<(LedgeraRequestInputProposal, SignatureEntry)>,
    pub vfun_sender:
        tokio::sync::mpsc::Sender<(LedgeraVoteFunctionInstanceDeclaration, SignatureEntry)>,
    pub vins_sender: tokio::sync::mpsc::Sender<(LedgeraVoteIns, SignatureEntry)>,
    pub vout_sender: tokio::sync::mpsc::Sender<(LedgeraVoteFunctionOutput, SignatureEntry)>,
    pub delivered_txs_sender:
        tokio::sync::mpsc::Sender<(LedgeraTransactionDeliveryNotification, SignatureEntry)>,
    pub vstored_sender: tokio::sync::mpsc::Sender<(LedgeraVoteStored, SignatureEntry)>,
}

pub async fn make_subscriptions_for_voter_node<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    comm_api: &Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: &Arc<LAT>,
    senders: SendersForVoterNodeSubscriptions<LAT>,
) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
    let mut comm_api = comm_api.lock().await;

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraRequestFunctionInstanceProposal<LAT>>(
            comm_params,
            &LedgeraCorePublicationTopics::Rfun.get_publication_topic_str(service.as_ref()),
            senders.rfun_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to Rfun client requests for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraRequestInputProposal>(
            comm_params,
            &LedgeraCorePublicationTopics::Rin.get_publication_topic_str(service.as_ref()),
            senders.rin_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to Rin client requests for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraVoteFunctionInstanceDeclaration>(
            comm_params,
            &LedgeraCorePublicationTopics::Vfun.get_publication_topic_str(service.as_ref()),
            senders.vfun_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to Vfun votes for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraVoteIns>(
            comm_params,
            &LedgeraCorePublicationTopics::Vins.get_publication_topic_str(service.as_ref()),
            senders.vins_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to Varg votes for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraVoteFunctionOutput>(
            comm_params,
            &LedgeraCorePublicationTopics::Vout.get_publication_topic_str(service.as_ref()),
            senders.vout_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to Vout votes for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraTransactionDeliveryNotification>(
            comm_params,
            &LedgeraCorePublicationTopics::TransactionDelivery
                .get_publication_topic_str(service.as_ref()),
            senders.delivered_txs_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to delivered transactions for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    match comm_api
        .subscribe_to_topic_and_deserialize_as::<LedgeraVoteStored>(
            comm_params,
            &LedgeraCorePublicationTopics::Vsto.get_publication_topic_str(service.as_ref()),
            senders.vstored_sender,
        )
        .await
    {
        Ok(_) => {
            log::info!(
                "As {:?} : subscribed to Vstored votes for service '{:}'",
                LedgeraCoreRoles::VoterComputer,
                service.get_service_name()
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}
