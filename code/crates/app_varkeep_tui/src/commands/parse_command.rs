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

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, multispace1};
use nom::combinator::map;
use nom::error::ParseError;
use nom::sequence::preceded;
use nom::{IResult, Parser};

use crate::commands::tui_commands::LedgeraVarkeepServiceTuiCommand;

pub fn parse_ledgera_varkeep_tui_command<'a>(
    input: &'a str,
) -> Result<LedgeraVarkeepServiceTuiCommand, String> {
    match parse_ledgera_varkeep_tui_command_inner::<'a, nom::error::Error<&'a str>>(input) {
        Ok((_, cmd)) => Ok(cmd),
        Err(e) => Err(format!("{:?}", e)),
    }
}

fn parse_ledgera_varkeep_tui_command_inner<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, LedgeraVarkeepServiceTuiCommand, E> {
    let mut parser = alt((
        map(tag("exit"), |_| LedgeraVarkeepServiceTuiCommand::Exit),
        map(
            (
                preceded(
                    tag("locassign"),
                    preceded(multispace1::<&'a str, E>, alphanumeric1),
                ),
                preceded(multispace1, alphanumeric1),
            ),
            |(vn, vv)| LedgeraVarkeepServiceTuiCommand::AssignLocal(vn.to_string(), vv.to_string()),
        ),
        map(
            (
                preceded(
                    tag("gloassign"),
                    preceded(multispace1::<&'a str, E>, alphanumeric1),
                ),
                preceded(multispace1, alphanumeric1),
            ),
            |(vn, vv)| {
                LedgeraVarkeepServiceTuiCommand::AssignGlobal(vn.to_string(), vv.to_string())
            },
        ),
    ));
    parser.parse(input)
}
