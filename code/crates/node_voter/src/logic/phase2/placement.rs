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

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    marker::PhantomData,
};

use ledgera_pki::message::SignatureEntry;
use ledgera_types::app_template::{
    predicates::LedgeraOperationMultiArgumentsPredicate, template::LedgeraApplicationTemplate,
};
use ledgera_types::requests::rin::LedgeraRequestInputProposal;
use ledgera_types::{digest::LedgeraDigest, votes::vins::LedgeraVoteInsInputProposalReference};

/**
 * Object instantiated by an "executor" whenever it is required to perform "phase 2"/"collect_unknowns"
 * for a given function instance initiated by a given "Rfun" request.
 *
 * This object is given the responsability to:
 * - keep track of received valid "Rin" proposals for unknown inputs of our function instance
 * - validate unknown input assignments for our function instance
 * - try to construct a valid unknown input assignment for our function instance from the received "Rin" requests
 * **/
pub(crate) struct ArgumentsPlacementFinder<LAT: LedgeraApplicationTemplate> {
    // length of the input arguments array
    max_arg_index: u32,
    // previously received valid "Rin" requests (in the context of a given "Rfun")
    locally_validated_argument_proposals: HashMap<SignatureEntry, LedgeraRequestInputProposal>,
    // for all unknown inputs of given index, signatures of all "Rin" that can be used to fill-in the unknown input
    potential_args_by_unknown_index: HashMap<u32, HashSet<SignatureEntry>>,
    // ***
    phantom: PhantomData<LAT>,
}

enum ArgumentReference {
    StaticKnownArguments(usize),
    UnknownProposedByRin(SignatureEntry),
}

impl<LAT: LedgeraApplicationTemplate> ArgumentsPlacementFinder<LAT> {
    pub fn get_locally_validated_argument_proposals(
        &self,
    ) -> &HashMap<SignatureEntry, LedgeraRequestInputProposal> {
        &self.locally_validated_argument_proposals
    }

    pub fn new(max_arg_index: u32, unknown_arguments_indices: &BTreeSet<u32>) -> Self {
        Self {
            max_arg_index,
            locally_validated_argument_proposals: HashMap::new(),
            potential_args_by_unknown_index: unknown_arguments_indices
                .iter()
                .map(|idx| (*idx, HashSet::new()))
                .collect(),
            phantom: PhantomData,
        }
    }

    pub fn acknowledge_new_locally_valid_rin(
        &mut self,
        rin_sig_entry: SignatureEntry,
        rin: LedgeraRequestInputProposal,
    ) {
        for idx in &rin.argument_indices {
            let for_arg = self.potential_args_by_unknown_index.get_mut(idx).unwrap();
            for_arg.insert(rin_sig_entry.clone());
        }
        self.locally_validated_argument_proposals
            .insert(rin_sig_entry, rin);
    }

    pub fn verify_vins_proposed_unknowns_assignment_validity(
        &self,
        proposed_unknowns_assignment: &BTreeMap<u32, LedgeraVoteInsInputProposalReference>,
        global_predicate: &Option<LAT::GlobalPredicate>,
        // ***
        known_inputs_values: &HashMap<usize, LAT::Data>,
        cached_retrieved_data: &HashMap<LedgeraDigest, LAT::Data>,
    ) -> bool {
        let mut args_array = Vec::new();
        for idx in 0..self.max_arg_index {
            if let Some(unknown_proposal) = proposed_unknowns_assignment.get(&idx) {
                args_array.push(ArgumentReference::UnknownProposedByRin(
                    unknown_proposal.signature_of_rin.clone(),
                ));
            } else {
                args_array.push(ArgumentReference::StaticKnownArguments(idx as usize));
            }
        }
        self.verify_args_array_validity(
            &args_array,
            global_predicate,
            known_inputs_values,
            cached_retrieved_data,
        )
    }

    fn verify_args_array_validity(
        &self,
        args_array: &[ArgumentReference],
        global_predicate: &Option<LAT::GlobalPredicate>,
        // ***
        // concrete values of the known inputs that were retrieved beforehand
        known_inputs_values: &HashMap<usize, LAT::Data>,
        // cached data to avoid having to query the storage too often to retrieve values of
        // the unknown inputs that are proposed in the "Vins" we are verifying
        cached_retrieved_data: &HashMap<LedgeraDigest, LAT::Data>,
    ) -> bool {
        if let Some(pred) = global_predicate {
            let refs: Vec<&LAT::Data> = args_array
                .iter()
                .map(|arg_ref| match arg_ref {
                    ArgumentReference::StaticKnownArguments(idx) => {
                        known_inputs_values.get(idx).unwrap()
                    }
                    ArgumentReference::UnknownProposedByRin(signature_entry) => {
                        let rin = self
                            .locally_validated_argument_proposals
                            .get(signature_entry)
                            .unwrap();
                        let value = cached_retrieved_data
                            .get(&rin.input_data.v.data_digest)
                            .unwrap();
                        value
                    }
                })
                .collect();
            // if we have a global predicate, it must hold on the final array of arguments
            pred.is_valid_for(&refs).unwrap()
        } else {
            // if we don't have a global predicate, then we have in any case found a valid mapping
            true
        }
    }

    fn try_find_unknowns_assignment_rec(
        &self,
        // ***
        args_array: &mut Vec<ArgumentReference>,
        next_index_to_add: u32,
        global_predicate: &Option<LAT::GlobalPredicate>,
        // ***
        known_inputs_values: &HashMap<usize, LAT::Data>,
        cached_retrieved_data: &HashMap<LedgeraDigest, LAT::Data>,
    ) -> bool {
        if next_index_to_add == self.max_arg_index {
            // we have filled-in all arguments
            return self.verify_args_array_validity(
                args_array,
                global_predicate,
                known_inputs_values,
                cached_retrieved_data,
            );
        }
        if let Some(potential_rins_sigs) =
            self.potential_args_by_unknown_index.get(&next_index_to_add)
        {
            for potential_rin_sig in potential_rins_sigs {
                args_array.push(ArgumentReference::UnknownProposedByRin(
                    potential_rin_sig.clone(),
                ));
                if self.try_find_unknowns_assignment_rec(
                    args_array,
                    next_index_to_add + 1,
                    global_predicate,
                    // ***
                    known_inputs_values,
                    cached_retrieved_data,
                ) {
                    return true;
                } else {
                    args_array.pop();
                }
            }
            false
        } else {
            args_array.push(ArgumentReference::StaticKnownArguments(
                next_index_to_add as usize,
            ));
            self.try_find_unknowns_assignment_rec(
                args_array,
                next_index_to_add + 1,
                global_predicate,
                // ***
                known_inputs_values,
                cached_retrieved_data,
            )
        }
    }

    pub fn try_find_unknowns_assignment(
        &self,
        global_predicate: &Option<LAT::GlobalPredicate>,
        known_inputs_values: &HashMap<usize, LAT::Data>,
        cached_retrieved_data: &HashMap<LedgeraDigest, LAT::Data>,
    ) -> Option<BTreeMap<u32, LedgeraVoteInsInputProposalReference>> {
        let mut args_array = vec![];
        if self.try_find_unknowns_assignment_rec(
            &mut args_array,
            0,
            global_predicate,
            // ***
            known_inputs_values,
            cached_retrieved_data,
        ) {
            let mut proposed_unknowns_assignment: BTreeMap<
                u32,
                LedgeraVoteInsInputProposalReference,
            > = BTreeMap::new();
            for (arg_index, arg_ref) in args_array.into_iter().enumerate() {
                if let ArgumentReference::UnknownProposedByRin(rin_sig) = arg_ref {
                    let rin = self
                        .locally_validated_argument_proposals
                        .get(&rin_sig)
                        .unwrap();
                    let vins_arg_ref = LedgeraVoteInsInputProposalReference::new(
                        rin_sig,
                        rin.argument_indices.clone(),
                        rin.input_data.clone(),
                    );
                    proposed_unknowns_assignment.insert(arg_index as u32, vins_arg_ref);
                }
            }
            Some(proposed_unknowns_assignment)
        } else {
            None
        }
    }
}
