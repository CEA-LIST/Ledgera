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
use nom::combinator::map;
use nom::error::ParseError;
use nom::IResult;
use nom::Parser;

use crate::commands::tui_commands::LedgeraServiceTemplateTuiCommand;

pub fn parse_ledgera_service_template_tui_command(
    input: &str,
) -> Result<LedgeraServiceTemplateTuiCommand, String> {
    match parse_inner::<nom::error::Error<&str>>(input) {
        Ok((_, cmd)) => Ok(cmd),
        Err(e) => Err(format!("{:?}", e)),
    }
}

fn parse_inner<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, LedgeraServiceTemplateTuiCommand, E> {
    let mut parser = alt((
        map(tag("exit"), |_| LedgeraServiceTemplateTuiCommand::Exit),
        // TODO : fill-in to parse other kinds of commands
    ));
    parser.parse(input)
}
