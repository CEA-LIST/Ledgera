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

use ledgera_pki::manager::SerdeSerializable64BitsSignature;

pub trait LedgeraOperationSingularArgumentPredicate<DataValue, RuntimeError> {
    /**
     * Given a computation instance with unknown arguments, clients
     * may submit proposals for these unkwnown arguments.
     * In the computation specification, we may specify predicates for
     * checking the validity of these argument proposals individually
     * (e.g., an integer must be greater than a value etc.)
     * **/
    fn is_valid_for(
        &self,
        value: &DataValue,
        function_instance_identifier: &SerdeSerializable64BitsSignature,
    ) -> Result<bool, RuntimeError>;
}

pub trait LedgeraOperationMultiArgumentsPredicate<DataValue, RuntimeError> {
    /**
     * Given a computation instance with unknown arguments, clients
     * may submit proposals for these unkwnown arguments.
     * In the computation specification, we may specify predicates for
     * checking the validity and compatibility of a set of arguments
     * (e.g., they must all be distinct from one another etc.)
     * **/
    fn is_valid_for(&self, arguments: &[&DataValue]) -> Result<bool, RuntimeError>;
}
