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

use ledgera_node_client::io::parser::LedgeraComputationItemsParser;
use ledgera_types::app_template::operation::LedgeraAtomicOperation;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};

use crate::commands::tui_commands::{
    LedgeraTuiCommand, LedgeraTuiCommandOperationArgument, LedgeraTuiCommandValueReference,
    LedgeraTuiExecuteCommand,
};

pub fn parse_ledgera_tui_command<
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &str,
) -> Result<LedgeraTuiCommand<LAT>, String> {
    let input = input.trim();
    match parse_ledgera_tui_command_inner::<'_, nom::error::Error<&'_ str>, LAT, CmpParser>(input) {
        Ok((_, cmd)) => Ok(cmd),
        Err(e) => Err(format!("{:?}", e)),
    }
}

fn parse_ledgera_tui_command_inner<
    'a,
    E: ParseError<&'a str>,
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &'a str,
) -> IResult<&'a str, LedgeraTuiCommand<LAT>, E> {
    let mut parser = alt((
        map(tag("exit"), |_| LedgeraTuiCommand::Exit),
        map(tag("print_graph"), |_| LedgeraTuiCommand::PrintGraph),
        map(
            (
                preceded((tag("rename"), multispace1), alphanumeric1),
                preceded(multispace1, alphanumeric1),
            ),
            |(m1, m2): (&'a str, &'a str)| {
                LedgeraTuiCommand::Rename(m1.to_string(), m2.to_string())
            },
        ),
        map(
            preceded((tag("get_value"), multispace1), alphanumeric1),
            |moniker: &'a str| LedgeraTuiCommand::GetValue(moniker.to_string()),
        ),
        map(
            preceded((tag("audit_value"), multispace1), |x| {
                parse_value_reference::<'a, E, LAT, CmpParser>(x)
            }),
            |val_ref| LedgeraTuiCommand::AuditValue(val_ref),
        ),
        map(
            preceded((tag("audit_comp"), multispace1), alphanumeric1),
            |comp_moniker: &'a str| LedgeraTuiCommand::AuditComputation(comp_moniker.to_string()),
        ),
        |x| parse_declare_atomic_operation_command::<'a, E, LAT, CmpParser>(x),
        |x| parse_push_arg_command::<'a, E, LAT>(x),
    ));
    parser.parse(input)
}

fn parse_declare_atomic_tag_operation<
    'a,
    E: ParseError<&'a str>,
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &'a str,
) -> IResult<&'a str, LedgeraAtomicOperation<LAT::Tag, LAT::Computation>, E> {
    let mut parser = map(
        preceded((tag("|"), multispace0), |x| {
            CmpParser::parse_tag_operation(x)
        }),
        LedgeraAtomicOperation::TagInputs,
    );
    parser.parse(input)
}

fn parse_declare_atomic_compute_operation<
    'a,
    E: ParseError<&'a str>,
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &'a str,
) -> IResult<&'a str, LedgeraAtomicOperation<LAT::Tag, LAT::Computation>, E> {
    let mut parser = map(
        (
            preceded(tag("/"), |x| CmpParser::parse_computation_operation(x)),
            // optional " -s"
            map(opt(preceded(multispace1, tag("-s"))), |opt_flag| {
                opt_flag.is_some()
            }),
        ),
        |(x, y)| LedgeraAtomicOperation::ComputeOutput {
            is_output_persistent: y,
            comp: x,
        },
    );
    parser.parse(input)
}

fn parse_declare_atomic_operation_command<
    'a,
    E: ParseError<&'a str>,
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &'a str,
) -> IResult<&'a str, LedgeraTuiCommand<LAT>, E> {
    let mut parser = map(
        (
            alt((
                |x| parse_declare_atomic_tag_operation::<'a, E, LAT, CmpParser>(x),
                |x| parse_declare_atomic_compute_operation::<'a, E, LAT, CmpParser>(x),
            )),
            // "*args"
            preceded(
                multispace1,
                separated_list1(multispace1, |x| {
                    parse_declared_computation_argument::<'a, E, LAT, CmpParser>(x)
                }),
            ),
            // optional global predicate "[P]"
            opt(delimited(
                preceded(multispace1, tag("[")),
                |x| CmpParser::parse_global_predicate(x),
                preceded(multispace0, tag("]")),
            )),
            // optional " -n"
            opt(preceded(
                (multispace1, tag("-n"), multispace1),
                alphanumeric1,
            )),
        ),
        |(operation, args, opt_global_pred, opt_name)| {
            let exec_comm = LedgeraTuiExecuteCommand::new(
                opt_name.map(|x| x.to_string()),
                operation,
                args,
                opt_global_pred,
            );
            LedgeraTuiCommand::Execute(exec_comm)
        },
    );
    parser.parse(input)
}

fn parse_push_arg_command<'a, E: ParseError<&'a str>, LAT: LedgeraApplicationTemplate>(
    input: &'a str,
) -> IResult<&'a str, LedgeraTuiCommand<LAT>, E> {
    let mut parser = map(
        (
            preceded((tag("push_arg"), multispace1), alphanumeric1),
            preceded(
                multispace1,
                delimited(
                    tag("{"),
                    separated_list1(tag(","), map(digit1, |x| str::parse::<u32>(x).unwrap())),
                    tag("}"),
                ),
            ),
            preceded(multispace1, preceded(tag("@"), alphanumeric1)),
        ),
        |(m, index, arg)| LedgeraTuiCommand::PushArg {
            comp_moniker: m.to_string(),
            arg_potential_indices: index.into_iter().collect(),
            data_moniker: arg.to_owned(),
        },
    );
    parser.parse(input)
}

fn parse_value_reference<
    'a,
    E: ParseError<&'a str>,
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &'a str,
) -> IResult<&'a str, LedgeraTuiCommandValueReference<LAT>, E> {
    let mut parser = alt((
        map(preceded(tag("@"), alphanumeric1), |x: &'a str| {
            LedgeraTuiCommandValueReference::ShorthandAsStorageReference(x.to_string())
        }),
        map(preceded(tag("*"), alphanumeric1), |x: &'a str| {
            LedgeraTuiCommandValueReference::ShorthandAsRawValue(x.to_string())
        }),
        map(
            (
                alt((map(tag("^"), |_| true), map(tag(""), |_| false))),
                |x| CmpParser::parse_value(x),
            ),
            |(is_input_persistent, value)| LedgeraTuiCommandValueReference::RawValue {
                is_input_persistent,
                value,
            },
        ),
    ));
    parser.parse(input)
}

fn parse_declared_computation_argument<
    'a,
    E: ParseError<&'a str>,
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
>(
    input: &'a str,
) -> IResult<&'a str, LedgeraTuiCommandOperationArgument<LAT>, E> {
    let mut parser = alt((
        map(
            |x| parse_value_reference::<'a, E, LAT, CmpParser>(x),
            |y| LedgeraTuiCommandOperationArgument::Value(y),
        ),
        map(
            |x| CmpParser::parse_local_predicate(x),
            /*delimited(
                tag("("),
                |x| CmpParser::parse_predicate(x),
                tag(")")
            ),*/
            |p| LedgeraTuiCommandOperationArgument::Predicate(p),
        ),
    ));
    parser.parse(input)
}
