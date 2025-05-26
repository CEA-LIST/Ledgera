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

use std::collections::{BTreeMap, HashMap};

use ledgera_pki::{manager::SerdeSerializable64BitsSignature, quorum::QuorumOfSignatures};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::app_template::{input::LedgeraInputArgument, operation::LedgeraAtomicOperation};
use ledgera_types::digest::LedgeraDigest;
use ledgera_types::proofs::proof_of_declaration::ProofOfFunctionDeclaration;
use ledgera_types::proofs::proof_of_integrity::ProofOfOperationIntegrity;
use ledgera_types::proofs::proof_of_storage::ProofOfShipmentToStorage;
use ledgera_types::proofs::proof_of_unknown_arguments_assignment_verification::ProofOfUnknownArgumentsAssignmentVerification;
use ledgera_types::votes::{
    vins::LedgeraVoteInsInputProposalReference, vout::LedgeraFunctionInstanceOutputKind,
};

use crate::know::LedgeraKnowledgeRepresentation;

pub fn get_ledgera_knowledge_json_representation<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    data_monikers: &HashMap<LedgeraDigest, String>,
    function_instance_monikers: &HashMap<SerdeSerializable64BitsSignature, String>,
) -> serde_json::Value {
    serde_json::json!(
        {
            "data" : get_ledgera_data_json_representation(
                know,
                data_monikers,
                function_instance_monikers
            ),
            "function instances" : get_ledgera_function_instances_json_representation(
                know,
                function_instance_monikers
            ),
        }
    )
}

fn get_ledgera_data_json_representation<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    data_monikers: &HashMap<LedgeraDigest, String>,
    function_instance_monikers: &HashMap<SerdeSerializable64BitsSignature, String>,
) -> serde_json::Value {
    let mut datas = serde_json::Map::new();
    let items = {
        let mut items: Vec<(String, LedgeraDigest)> = know
            .per_data_value
            .keys()
            .cloned()
            .map(|x| (data_monikers.get(&x).unwrap().clone(), x))
            .collect();
        items.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));
        items
    };
    for (moniker, digest) in items {
        datas.insert(
            moniker,
            get_specific_data_json(know, function_instance_monikers, digest),
        );
    }
    serde_json::Value::Object(datas)
}

fn get_specific_data_json<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    function_instance_monikers: &HashMap<SerdeSerializable64BitsSignature, String>,
    digest: LedgeraDigest,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    let data_k = know.per_data_value.get(&digest).unwrap();
    attributes.insert(
        "1:digest".to_string(),
        serde_json::Value::String(digest.to_hexadecimal_string()),
    );
    match &data_k.data_value {
        Some(v) => {
            attributes.insert(
                "2:value".to_string(),
                serde_json::Value::String(format!("{:?}", v)),
            );
        }
        None => {
            // ***
        }
    }
    {
        let mut json_poss = serde_json::Map::new();
        for ((fid, pdk), poss) in data_k.proofs_of_storage.iter() {
            let c_moniker = function_instance_monikers.get(fid).unwrap();
            for (count, pos) in (1..).zip(poss.iter()) {
                json_poss.insert(
                    format!("{:?}@{:}#{}", pdk, c_moniker, count),
                    get_proof_of_storage_json(pos),
                );
            }
        }
        attributes.insert("3:LP_S".to_string(), serde_json::Value::Object(json_poss));
    }
    serde_json::Value::Object(attributes)
}

fn get_ledgera_function_instances_json_representation<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    function_instance_monikers: &HashMap<SerdeSerializable64BitsSignature, String>,
) -> serde_json::Value {
    let mut funcs = serde_json::Map::new();
    let items = {
        let mut items: Vec<(String, SerdeSerializable64BitsSignature)> = know
            .per_function_instance
            .keys()
            .cloned()
            .map(|x| {
                let fun_moniker = match function_instance_monikers.get(&x) {
                    None => "?".to_string(),
                    Some(m) => m.clone(),
                };
                (fun_moniker, x)
            })
            .collect();
        items.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));
        items
    };
    for (moniker, function_instance_id) in items {
        funcs.insert(
            moniker,
            get_specific_function_instance_json(know, function_instance_id),
        );
    }
    serde_json::Value::Object(funcs)
}

fn get_specific_function_instance_json<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    function_instance_id: SerdeSerializable64BitsSignature,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    let func_k = know
        .per_function_instance
        .get(&function_instance_id)
        .unwrap();
    attributes.insert(
        "1:function_instance_id".to_string(),
        serde_json::Value::String(function_instance_id.to_hexadecimal_string()),
    );
    match &func_k.spec {
        Some(spec) => {
            match &spec.operation {
                LedgeraAtomicOperation::TagInputs(tag) => {
                    attributes.insert(
                        "2:identity function tag".to_string(),
                        serde_json::Value::String(format!("{:?}", tag)),
                    );
                }
                LedgeraAtomicOperation::ComputeOutput {
                    is_output_persistent,
                    comp,
                } => {
                    attributes.insert(
                        "2:compute function".to_string(),
                        serde_json::Value::String(format!("{:?}", comp)),
                    );
                    attributes.insert(
                        "2:is_output_persistent".to_string(),
                        serde_json::Value::String(format!("{:?}", is_output_persistent)),
                    );
                }
            }
            if let Some(glo_pred) = &spec.global_arguments_predicate {
                attributes.insert(
                    "3:global_predicate".to_string(),
                    serde_json::Value::String(format!("{:?}", glo_pred)),
                );
            }
            {
                let mut json_args = serde_json::Map::new();
                for (index, arg) in spec.arguments.iter().enumerate() {
                    json_args.insert(
                        format!("@index{}", index),
                        get_in_spec_input_specification_json::<LAT>(arg),
                    );
                }
                attributes.insert(
                    "4:arguments_specification".to_string(),
                    serde_json::Value::Object(json_args),
                );
            }
        }
        None => {
            // ***
        }
    }
    {
        let mut json_pods = vec![];
        for pod in func_k.proofs_of_declaration.iter() {
            json_pods.push(get_proof_of_declaration_json(pod));
        }
        attributes.insert("5:LP_FD".to_string(), serde_json::Value::Array(json_pods));
    }
    if let Some(pouav) = &func_k.agreed_upon_unknowns {
        attributes.insert("6:LP_VIA".to_string(), get_proof_of_assignment_json(pouav));
    }
    {
        let mut json_pois = vec![];
        for poi in func_k.proofs_of_result_integrity.iter() {
            json_pois.push(get_proof_of_integrity_json(know, poi));
        }
        attributes.insert("7:LP_C".to_string(), serde_json::Value::Array(json_pois));
    }
    if let Some(got_res) = &func_k.result_kind {
        attributes.insert(
            "8:result_kind".to_string(),
            get_result_kind_json(know, got_res),
        );
    }
    serde_json::Value::Object(attributes)
}

fn get_result_kind_json<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    result_kind: &LedgeraFunctionInstanceOutputKind,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    match result_kind {
        LedgeraFunctionInstanceOutputKind::ComputedOutput {
            is_output_persistent,
            output_digest,
        } => {
            attributes.insert(
                "1:output_digest".to_string(),
                serde_json::Value::String(output_digest.to_hexadecimal_string()),
            );
            attributes.insert(
                "2:is_output_persistent".to_string(),
                serde_json::Value::String(format!("{}", is_output_persistent)),
            );
            if let Some(data_k) = know.per_data_value.get(output_digest) {
                if let Some(v) = &data_k.data_value {
                    attributes.insert(
                        "3:output_value".to_string(),
                        serde_json::Value::String(format!("{:?}", v)),
                    );
                }
            }
        }
        LedgeraFunctionInstanceOutputKind::TaggedInputs => {
            attributes.insert(
                "1:tag".to_string(),
                serde_json::Value::String(
                    "the function does not compute an output (it is the identity)".to_string(),
                ),
            );
        }
    }
    serde_json::Value::Object(attributes)
}

fn get_in_spec_input_specification_json<LAT: LedgeraApplicationTemplate>(
    arg: &LedgeraInputArgument<LAT::Data, LAT::LocalPredicate>,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    match arg {
        LedgeraInputArgument::RawValue {
            is_input_persistent,
            value,
        } => {
            attributes.insert(
                "1:input_kind".to_string(),
                serde_json::Value::String("RawValue".to_string()),
            );
            attributes.insert(
                "2:value".to_string(),
                serde_json::Value::String(format!("{:?}", value)),
            );
            attributes.insert(
                "3:is_input_persistent".to_string(),
                serde_json::Value::String(format!("{:?}", is_input_persistent)),
            );
        }
        LedgeraInputArgument::ReferenceToStorage(pos) => {
            attributes.insert(
                "1:input_kind".to_string(),
                serde_json::Value::String("ReferenceToStorage".to_string()),
            );
            attributes.insert("2:pos".to_string(), get_proof_of_storage_json(pos));
        }
        LedgeraInputArgument::Unknown(pred) => {
            attributes.insert(
                "1:input_kind".to_string(),
                serde_json::Value::String("Unknown".to_string()),
            );
            attributes.insert(
                "2:local_predicate".to_string(),
                serde_json::Value::String(format!("{:?}", pred)),
            );
        }
    }
    serde_json::Value::Object(attributes)
}

fn get_proof_of_storage_json(pos: &ProofOfShipmentToStorage) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "1:proof_kind".to_string(),
        serde_json::Value::String("LP_S".to_string()),
    );
    attributes.insert(
        "2:function_instance".to_string(),
        serde_json::Value::String(pos.v.function_instance_identifier.to_hexadecimal_string()),
    );
    attributes.insert(
        "3:stored_as".to_string(),
        serde_json::Value::String(format!("{:}", pos.v.data_kind)),
    );
    attributes.insert(
        "4:value_digest".to_string(),
        serde_json::Value::String(pos.v.data_digest.to_hexadecimal_string()),
    );
    attributes.insert("5:quorum".to_string(), get_signatures_quorum_json(&pos.q));
    serde_json::Value::Object(attributes)
}

fn get_proof_of_declaration_json(pod: &ProofOfFunctionDeclaration) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "1:proof_kind".to_string(),
        serde_json::Value::String("LP_FD".to_string()),
    );
    attributes.insert(
        "2:function_instance".to_string(),
        serde_json::Value::String(pod.v.function_instance_identifier.to_hexadecimal_string()),
    );
    {
        let mut known_args_json = serde_json::Map::new();
        for (i, a) in pod.v.known_arguments.iter() {
            known_args_json.insert(
                format!("@{}", i),
                serde_json::Value::String(a.to_hexadecimal_string()),
            );
        }
        attributes.insert(
            "3:known_arguments".to_string(),
            serde_json::Value::Object(known_args_json),
        );
    }
    attributes.insert(
        "4:unknown_arguments_indices".to_string(),
        serde_json::Value::String(format!("{:?}", pod.v.unknown_arguments_indices)),
    );
    attributes.insert(
        "5:persistent_inputs_indices".to_string(),
        serde_json::Value::String(format!("{:?}", pod.v.persistent_inputs_indices)),
    );
    attributes.insert("6:quorum".to_string(), get_signatures_quorum_json(&pod.q));
    serde_json::Value::Object(attributes)
}

fn get_proof_of_assignment_json(
    pouav: &ProofOfUnknownArgumentsAssignmentVerification,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "1:proof_kind".to_string(),
        serde_json::Value::String("LP_VIA".to_string()),
    );
    attributes.insert(
        "2:function_instance".to_string(),
        serde_json::Value::String(pouav.v.function_instance_identifier.to_hexadecimal_string()),
    );
    attributes.insert(
        "3:assignment".to_string(),
        get_assignment_json(&pouav.v.proposed_unknowns_assignment),
    );
    attributes.insert("3:quorum".to_string(), get_signatures_quorum_json(&pouav.q));
    serde_json::Value::Object(attributes)
}

fn get_assignment_json(
    assignment: &BTreeMap<u32, LedgeraVoteInsInputProposalReference>,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    for (index, ass) in assignment {
        let mut ass_json = serde_json::Map::new();
        ass_json.insert(
            "1:proposed_by".to_string(),
            serde_json::Value::String(hex::encode(
                ass.signature_of_rin.serialized_signing_public_key,
            )),
        );
        ass_json.insert(
            "2:at_indices".to_string(),
            serde_json::Value::String(format!("{:?}", ass.argument_indices)),
        );
        ass_json.insert(
            "3:proposed_value_proof_of_storage".to_string(),
            get_proof_of_storage_json(&ass.pos),
        );
        ass_json.insert(
            "4:signature_of_input_proposal".to_string(),
            serde_json::Value::String(
                ass.signature_of_rin
                    .serializable_signature
                    .to_hexadecimal_string(),
            ),
        );
        attributes.insert(
            format!("@index{}", index),
            serde_json::Value::Object(ass_json),
        );
    }
    serde_json::Value::Object(attributes)
}

fn get_proof_of_integrity_json<LAT: LedgeraApplicationTemplate>(
    know: &LedgeraKnowledgeRepresentation<LAT>,
    poi: &ProofOfOperationIntegrity,
) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "1:proof_kind".to_string(),
        serde_json::Value::String("LP_C".to_string()),
    );
    attributes.insert(
        "2:function_instance".to_string(),
        serde_json::Value::String(poi.v.function_instance_identifier.to_hexadecimal_string()),
    );
    if let Some(agg_ref_digest) = &poi.v.unknowns_agreement_ref {
        attributes.insert(
            "3:pouav_digest".to_string(),
            serde_json::Value::String(agg_ref_digest.to_hexadecimal_string()),
        );
    }
    attributes.insert(
        "4:result_kind".to_string(),
        get_result_kind_json(know, &poi.v.result_kind),
    );
    attributes.insert("5:quorum".to_string(), get_signatures_quorum_json(&poi.q));
    serde_json::Value::Object(attributes)
}

fn get_signatures_quorum_json(q: &QuorumOfSignatures) -> serde_json::Value {
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "1:signed_data".to_string(),
        serde_json::Value::String(hex::encode(&q.agreed_upon_value)),
    );
    {
        let mut signatures = serde_json::Map::new();
        for (i, sig) in q.signatures.iter().enumerate() {
            signatures.insert(
                format!("s{}_key", i),
                serde_json::Value::String(hex::encode(sig.serialized_signing_public_key)),
            );
            signatures.insert(
                format!("s{}_sig", i),
                serde_json::Value::String(sig.serializable_signature.to_hexadecimal_string()),
            );
        }
        attributes.insert(
            "2:signatures".to_string(),
            serde_json::Value::Object(signatures),
        );
    }
    serde_json::Value::Object(attributes)
}
