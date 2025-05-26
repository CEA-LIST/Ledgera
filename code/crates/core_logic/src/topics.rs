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

use ledgera_types::app_template::template::LedgeraApplicationTemplate;

pub enum LedgeraCorePublicationTopics {
    Rsto,
    Rfun,
    Rin,
    Vsto,
    Vfun,
    Vins,
    Vout,
    Nout(String),
    TransactionSubmission,
    TransactionDelivery,
}

impl LedgeraCorePublicationTopics {
    pub fn get_publication_topic_str<LAT: LedgeraApplicationTemplate>(
        &self,
        template: &LAT,
    ) -> String {
        match self {
            LedgeraCorePublicationTopics::Rfun => {
                format!("{}/Rfun", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Rin => {
                format!("{}/Rin", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Rsto => {
                format!("{}/Rsto", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Vfun => {
                format!("{}/Vfun", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Vins => {
                format!("{}/Vins", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Vout => {
                format!("{}/Vout", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Vsto => {
                format!("{}/Vsto", template.get_service_name())
            }
            LedgeraCorePublicationTopics::Nout(x) => {
                format!("{}/Nout{}", template.get_service_name(), x)
            }
            LedgeraCorePublicationTopics::TransactionSubmission => {
                format!("{}/TransactionSubmission", template.get_service_name())
            }
            LedgeraCorePublicationTopics::TransactionDelivery => {
                format!("{}/TransactionDelivery", template.get_service_name())
            }
        }
    }
}

pub enum LedgeraCoreQueryTopics {
    Value,
    Audit,
}

impl LedgeraCoreQueryTopics {
    pub fn get_query_topic_str<LAT: LedgeraApplicationTemplate>(&self, template: &LAT) -> String {
        match self {
            LedgeraCoreQueryTopics::Value => format!("{}/QVal", template.get_service_name()),
            LedgeraCoreQueryTopics::Audit => format!("{}/QAud", template.get_service_name()),
        }
    }
}
