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

//! The provided behavior for apps where high-level commands map directly onto Ledgera core
//! function instances (declare a computation, push an argument, audit, retrieve a value, ...).

use std::marker::PhantomData;

use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_knowledge_representation::printer::LedgeraComputationItemsPrinter;
use ledgera_node_client::client_logic::client_behavior::LedgeraClientRunOutput;
use ledgera_node_client::io::parser::LedgeraComputationItemsParser;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::input::LedgeraInputArgument;
use ledgera_types::app_template::spec::LedgeraAtomicOperationSpecification;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::digest::LedgeraDigest;
use ledgera_types::messages::qaud::LedgeraQueryAudit;
use ledgera_types::requests::rin::LedgeraRequestInputProposal;

use crate::behavior::{LedgeraTuiBehavior, TuiBackgroundEvent, TuiControlFlow};
use crate::commands::builtin::print_graph::exec_print_graph;
use crate::commands::parse_command::parse_ledgera_tui_command;
use crate::commands::tui_commands::{
    LedgeraTuiCommand, LedgeraTuiCommandOperationArgument, LedgeraTuiCommandValueReference,
};
use crate::doc::DocumentedCmpTying;
use crate::knowledge::tui_knowledge::LedgeraTuiKnowledge;

/// Behavior that drives the Ledgera core runtime directly from the generic command language.
pub struct FunctionInstanceBehavior<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate + DocumentedCmpTying,
    CmpParser: LedgeraComputationItemsParser<LAT>,
    CmpPrinter: LedgeraComputationItemsPrinter<LAT>,
> {
    client_node_run: LedgeraClientRunOutput<PKI, Sess, LAT>,
    phantom: PhantomData<(CmpParser, CmpPrinter)>,
}

impl<
        PKI: PublicKeyInfrastructure,
        Sess: PubSubNetwork,
        LAT: LedgeraApplicationTemplate + DocumentedCmpTying,
        CmpParser: LedgeraComputationItemsParser<LAT>,
        CmpPrinter: LedgeraComputationItemsPrinter<LAT>,
    > FunctionInstanceBehavior<PKI, Sess, LAT, CmpParser, CmpPrinter>
{
    pub fn new(client_node_run: LedgeraClientRunOutput<PKI, Sess, LAT>) -> Self {
        Self {
            client_node_run,
            phantom: PhantomData,
        }
    }
}

impl<
        PKI: PublicKeyInfrastructure,
        Sess: PubSubNetwork,
        LAT: LedgeraApplicationTemplate + DocumentedCmpTying,
        CmpParser: LedgeraComputationItemsParser<LAT>,
        CmpPrinter: LedgeraComputationItemsPrinter<LAT>,
    > LedgeraTuiBehavior for FunctionInstanceBehavior<PKI, Sess, LAT, CmpParser, CmpPrinter>
{
    type App = LAT;
    type Command = LedgeraTuiCommand<LAT>;

    fn service_doc(&self) -> &'static str {
        LAT::get_doc()
    }

    fn parse_command(input: &str) -> Result<Self::Command, String> {
        parse_ledgera_tui_command::<LAT, CmpParser>(input).map_err(|e| e.to_string())
    }

    fn check_monikers(
        cmd: &Self::Command,
        knowledge: &LedgeraTuiKnowledge<Self::App>,
    ) -> Result<(), String> {
        cmd.check_monikers(knowledge)
            .map_err(|e| format!("{:?}", e))
    }

    async fn handle_command(
        &mut self,
        cmd: Self::Command,
        knowledge: &mut LedgeraTuiKnowledge<Self::App>,
        node_name: &str,
    ) -> TuiControlFlow {
        match cmd {
            LedgeraTuiCommand::Exit => {
                return TuiControlFlow::ReturnToMenu;
            }
            LedgeraTuiCommand::PrintGraph => {
                let now = chrono::Local::now();
                let graph_file_name = format!(
                    "./testnet/{}/kgraph_{}",
                    node_name,
                    now.format("%Y_%m_%d_%H_%M_%S")
                );
                exec_print_graph::<LAT, CmpPrinter>(
                    &graph_file_name,
                    &knowledge.cached_client_knowledge,
                    &knowledge.computations_monikers,
                    &knowledge.data_monikers,
                );
            }
            LedgeraTuiCommand::Rename(m1, m2) => {
                knowledge.rename_moniker(&m1, m2).unwrap();
            }
            LedgeraTuiCommand::AuditValue(valref) => {
                let value_digest: LedgeraDigest = match valref {
                    LedgeraTuiCommandValueReference::RawValue {
                        is_input_persistent: _,
                        value,
                    } => LedgeraDigest::from_serializable(&value).unwrap(),
                    LedgeraTuiCommandValueReference::ShorthandAsStorageReference(moniker) => {
                        knowledge.data_monikers.get(&moniker).unwrap().clone()
                    }
                    LedgeraTuiCommandValueReference::ShorthandAsRawValue(moniker) => {
                        knowledge.data_monikers.get(&moniker).unwrap().clone()
                    }
                };
                match self
                    .client_node_run
                    .core_runtime
                    .audit_log(LedgeraQueryAudit::StoredValue(value_digest))
                    .await
                {
                    Ok(audit_response) => {
                        for t in audit_response {
                            log::warn!(
                                "TUI : as a response to value audit, received transaction {:?}",
                                t
                            );
                        }
                    }
                    Err(e) => {
                        log::warn!("TUI : could not send user request with error {:?}", e);
                    }
                }
            }
            LedgeraTuiCommand::AuditComputation(comp_moniker) => {
                let comp_id = knowledge
                    .computations_monikers
                    .get(&comp_moniker)
                    .unwrap()
                    .clone();
                match self
                    .client_node_run
                    .core_runtime
                    .audit_log(LedgeraQueryAudit::Computation(comp_id))
                    .await
                {
                    Ok(audit_response) => {
                        for t in audit_response {
                            log::warn!(
                                "TUI : as a response to function audit, received transaction {:?}",
                                t
                            );
                        }
                    }
                    Err(e) => {
                        log::warn!("TUI : could not send user request with error {:?}", e);
                    }
                }
            }
            LedgeraTuiCommand::GetValue(m) => {
                let value_digest = knowledge.data_monikers.get(&m).unwrap().clone();
                let pos = match knowledge
                    .cached_client_knowledge
                    .per_data_value
                    .get(&value_digest)
                {
                    None => None,
                    Some(x) => x
                        .proofs_of_storage
                        .values()
                        .next()
                        .and_then(|z| z.iter().next().cloned()),
                };
                match self
                    .client_node_run
                    .core_runtime
                    .retrieve_data(value_digest.clone(), pos)
                    .await
                {
                    Ok(Some(value)) => {
                        knowledge.update_on_user_retrieved_raw_value(value_digest, value);
                        log::info!("TUI : received a value following data request on digest : updating knowledge");
                    }
                    Ok(None) => {
                        log::warn!("TUI : received absence of value from storage");
                    }
                    Err(e) => {
                        log::warn!("TUI : could not send user request with error {:?}", e);
                    }
                }
            }
            LedgeraTuiCommand::Execute(exec_comm) => {
                let mut ledgera_args = vec![];
                for tui_arg in &exec_comm.arguments {
                    let ledgera_arg = match tui_arg {
                        LedgeraTuiCommandOperationArgument::Predicate(p) => {
                            LedgeraInputArgument::Unknown(p.clone())
                        }
                        LedgeraTuiCommandOperationArgument::Value(tui_v_ref) => match tui_v_ref {
                            LedgeraTuiCommandValueReference::RawValue {
                                is_input_persistent,
                                value,
                            } => LedgeraInputArgument::RawValue {
                                is_input_persistent: *is_input_persistent,
                                value: value.clone(),
                            },
                            LedgeraTuiCommandValueReference::ShorthandAsStorageReference(
                                data_moniker,
                            ) => {
                                let data_digest =
                                    knowledge.data_monikers.get(data_moniker).unwrap();
                                let pos = knowledge
                                    .cached_client_knowledge
                                    .per_data_value
                                    .get(data_digest)
                                    .unwrap()
                                    .proofs_of_storage
                                    .iter()
                                    .next()
                                    .unwrap()
                                    .1
                                    .iter()
                                    .next()
                                    .unwrap()
                                    .clone();
                                LedgeraInputArgument::ReferenceToStorage(pos)
                            }
                            LedgeraTuiCommandValueReference::ShorthandAsRawValue(data_moniker) => {
                                let data_digest =
                                    knowledge.data_monikers.get(data_moniker).unwrap();
                                let v = knowledge
                                    .cached_client_knowledge
                                    .per_data_value
                                    .get(data_digest)
                                    .unwrap()
                                    .data_value
                                    .as_ref()
                                    .unwrap()
                                    .clone();
                                LedgeraInputArgument::RawValue {
                                    is_input_persistent: false,
                                    value: v,
                                }
                            }
                        },
                    };
                    ledgera_args.push(ledgera_arg);
                }
                log::info!(
                    "TUI : sent computation declaration request to client with {:?} args",
                    ledgera_args.len()
                );
                let comp_spec = LedgeraAtomicOperationSpecification::new(
                    exec_comm.operation,
                    exec_comm.opt_global_pred,
                    ledgera_args,
                );
                let comp_id_opt = match self
                    .client_node_run
                    .core_runtime
                    .compute_function(comp_spec)
                    .await
                {
                    Ok(fid) => Some(fid),
                    Err(e) => {
                        log::warn!("TUI : did not receive unique computation ID from client with error {:?}", e);
                        None
                    }
                };
                if let Some(fid) = comp_id_opt {
                    knowledge.update_on_user_proposed_function_instance(fid, exec_comm.opt_moniker);
                }
            }
            LedgeraTuiCommand::PushArg {
                comp_moniker,
                arg_potential_indices,
                data_moniker,
            } => {
                let comp_id = knowledge.computations_monikers.get(&comp_moniker).unwrap();
                let data_digest = knowledge.data_monikers.get(&data_moniker).unwrap();
                let pos = knowledge
                    .cached_client_knowledge
                    .per_data_value
                    .get(data_digest)
                    .unwrap()
                    .proofs_of_storage
                    .iter()
                    .next()
                    .unwrap()
                    .1
                    .iter()
                    .next()
                    .unwrap()
                    .clone();
                let rin_request = LedgeraRequestInputProposal::new(
                    comp_id.clone(),
                    arg_potential_indices.into_iter().collect(),
                    pos,
                );
                if let Err(e) = self
                    .client_node_run
                    .core_runtime
                    .propose_input(&rin_request)
                    .await
                {
                    log::warn!("TUI : could not send user request with error {:?}", e);
                }
            }
        }
        TuiControlFlow::Continue
    }

    async fn next_background_event(&mut self) -> TuiBackgroundEvent<Self::App> {
        match self
            .client_node_run
            .to_app_stream_of_validated_core_msgs
            .recv()
            .await
        {
            Some(feedback) => TuiBackgroundEvent::CoreFeedback(feedback),
            // channel closed: never resolve, so the engine relies on its other select branches
            None => std::future::pending().await,
        }
    }
}
