/*
 *   Copyright (c) 2022 R3BL LLC
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

// https://github.com/rust-lang/rust-clippy
// https://rust-lang.github.io/rust-clippy/master/index.html
#![warn(clippy::all)]
#![warn(clippy::unwrap_in_result)]
#![warn(rust_2018_idioms)]

/// Enable debug logging.
pub const ENABLE_TRACE_EXAMPLES: bool = true;

// Attach sources.
mod ex_app_no_layout;
mod ex_app_with_1col_layout;
mod ex_app_with_2col_layout;
mod ex_editor;
mod ex_pitch;
mod ex_rc;

// Use other crates.
use std::str::FromStr;

use crossterm::style::Stylize;
use r3bl_rs_utils_core::*;
use r3bl_terminal_async::*;
use r3bl_tui::*;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

#[tokio::main]
async fn main() -> CommonResult<()> {
    println!("{}", style_prompt(generate_help_msg().as_str()));

    // If the terminal is not fully interactive, then return early.
    let Some(mut terminal_async) = TerminalAsync::try_new("> ").await? else {
        return CommonError::new_err_with_only_msg("Terminal is not fully interactive");
    };

    // Pre-populate the readline's history with some entries.
    for command in AutoCompleteCommand::iter() {
        terminal_async
            .readline
            .add_history_entry(command.to_string());
    }

    loop {
        let result_readline_event = terminal_async.get_readline_event().await;
        match result_readline_event {
            Ok(readline_event) => match readline_event {
                ReadlineEvent::Line(input) => {
                    if run_user_selected_example(input).await.is_err() {
                        break;
                    };
                    crossterm::terminal::enable_raw_mode()?;
                }
                ReadlineEvent::Eof | ReadlineEvent::Interrupted => break,
                ReadlineEvent::Resized => { /* continue */ }
            },
            Err(_) => {
                break;
            }
        }
    }

    ok!()
}

/// You can type both "0" or "App with no layout" to run the first example. Here are some
/// details:
/// - `selection` is what the user types in the terminal, eg: "0" or "App with no layout".
/// - result_command is the parsed command from the selection, eg:
///   [AutoCompleteCommand::NoLayout].
///
/// # Raw mode caveat
///
/// This function will take the terminal out of raw mode when it returns. This is because
/// the examples below will use `r3bl_tui` which will put the terminal in raw mode, use
/// alt screen, and then restore it all when it exits.
async fn run_user_selected_example(selection: String) -> CommonResult<()> {
    let result_command /* Eg: Ok(Exit) */ =
        AutoCompleteCommand::from_str(&selection /* eg: "0" */);
    match result_command {
        Ok(command) => match command {
            AutoCompleteCommand::NoLayout => {
                throws!(ex_app_no_layout::launcher::run_app().await?)
            }
            AutoCompleteCommand::OneColLayout => {
                throws!(ex_app_with_1col_layout::launcher::run_app().await?)
            }
            AutoCompleteCommand::TwoColLayout => {
                throws!(ex_app_with_2col_layout::launcher::run_app().await?)
            }
            AutoCompleteCommand::Editor => {
                throws!(ex_editor::launcher::run_app().await?)
            }
            AutoCompleteCommand::Slides => {
                throws!(ex_pitch::launcher::run_app().await?)
            }
            AutoCompleteCommand::Commander => {
                throws!(ex_rc::launcher::run_app().await?)
            }
            AutoCompleteCommand::Exit => CommonError::new_err_with_only_msg("Exiting..."),
        },
        Err(_) => {
            println!("{} {}", "Invalid selection:".blue(), selection.red().bold());
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, EnumString, EnumIter, Display, AsRefStr)]
enum AutoCompleteCommand {
    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "App with no layout")]
    #[strum(serialize = "0")]
    NoLayout,

    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "App with 1 column responsive layout")]
    #[strum(serialize = "1")]
    OneColLayout,

    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "App with 2 column responsive layout")]
    #[strum(serialize = "2")]
    TwoColLayout,

    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "Markdown editor, syntax highlighting, modal dialog, and emoji")]
    #[strum(serialize = "3")]
    Editor,

    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "Why R3BL? Why TUI?")]
    #[strum(serialize = "4")]
    Slides,

    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "R3BL CMDR prototype")]
    #[strum(serialize = "5")]
    Commander,

    #[strum(ascii_case_insensitive)]
    #[strum(to_string = "Exit")]
    #[strum(serialize = "x")]
    Exit,
}

fn generate_help_msg() -> String {
    format!(
        "\
Welcome to the R3BL TUI demo app.
Type a number to run corresponding example:
  0. 📏 {}
  1. 📐 {}
  2. 📐 {}
  3. 🐒 {}
  4. ⚾ {}
  5. 📔 {}

or type Ctrl+C, Ctrl+D, 'exit', or 'x' to exit",
        AutoCompleteCommand::NoLayout,
        AutoCompleteCommand::OneColLayout,
        AutoCompleteCommand::TwoColLayout,
        AutoCompleteCommand::Editor,
        AutoCompleteCommand::Slides,
        AutoCompleteCommand::Commander,
    )
}
