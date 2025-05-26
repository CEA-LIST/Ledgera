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

use ledgera_app_string_concat::lat_binding::*;
use ledgera_node_client::io::parser::LedgeraComputationItemsParser;
use nom::bytes::tag;
use nom::character::complete::{alphanumeric1, digit1};
use nom::combinator::map;
use nom::error::ParseError;
use nom::sequence::{delimited, preceded};
use nom::IResult;
use nom::Parser;

pub struct StrConcatComputationParser {}

impl LedgeraComputationItemsParser<StrConcatBackend> for StrConcatComputationParser {
    fn parse_computation_operation<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, StrConcatComputation, E> {
        map(tag("concat"), |_| StrConcatComputation::Concat).parse(input)
    }
    fn parse_tag_operation<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, StrConcatTag, E> {
        map(tag("tag"), |_| StrConcatTag::Tag).parse(input)
    }

    fn parse_value<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, StrConcatData, E> {
        map(alphanumeric1, |s: &'a str| StrConcatData {
            string: s.to_string(),
        })
        .parse(input)
    }

    fn parse_local_predicate<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, StrConcatLocalPredicate, E> {
        map(
            delimited(tag("("), preceded(tag(">"), digit1), tag(")")),
            |s| StrConcatLocalPredicate::StringLongerThan(str::parse::<u32>(s).unwrap()),
        )
        .parse(input)
    }

    fn parse_global_predicate<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, StrConcatGlobalPredicate, E> {
        map(tag("distinct"), |_| {
            StrConcatGlobalPredicate::PairwiseDistinct
        })
        .parse(input)
    }
}
