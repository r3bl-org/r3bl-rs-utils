/*
 *   Copyright (c) 2023 R3BL LLC
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

use std::{io::{stdin, BufRead, Result},
          process::Command};

use clap::{Args, CommandFactory, Parser, Subcommand};
use crossterm::style::Stylize;
use r3bl_rs_utils_core::*;
use r3bl_tuify::*;
use StdinIsPipedResult::*;
use StdoutIsPipedResult::*;

const SELECTED_ITEM_SYMBOL: char = '%';

#[derive(Debug, Parser)]
#[command(bin_name = "rt")]
#[command(about = "Easily add lightweight TUI capabilities to any CLI apps using pipes", long_about = None)]
#[command(version)]
#[command(next_line_help = true)]
#[command(arg_required_else_help(true))]
pub struct AppArgs {
    #[clap(subcommand)]
    command: CLICommand,

    #[clap(flatten)]
    global_opts: GlobalOpts,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Print debug output to log file (log.txt)
    #[arg(long, short = 'l')]
    enable_logging: bool,

    /// Optional maximum height of the TUI (rows)
    #[arg(value_name = "height", long, short = 'r')]
    tui_height: Option<usize>,

    /// Optional maximum width of the TUI (columns)
    #[arg(value_name = "width", long, short = 'c')]
    tui_width: Option<usize>,
}

#[derive(Debug, Subcommand)]
enum CLICommand {
    /// Show TUI to allow you to select one or more options from a list, piped in via stdin 👉
    SelectFromList {
        /// Would you like to select one or more items?
        #[arg(value_name = "mode", long, short = 's')]
        selection_mode: SelectionMode,

        /// Each selected item is passed to this command as `%` and executed in your shell.
        /// For eg: "echo %". Please wrap the command in quotes 💡
        #[arg(value_name = "command", long, short = 'c')]
        command_to_run_with_selection: String,
    },
}

fn get_bin_name() -> String {
    let cmd = AppArgs::command();
    cmd.get_bin_name().unwrap_or("this command").to_string()
}

fn main() -> Result<()> {
    // If no args are passed, the following line will fail, and help will be printed
    // thanks to `arg_required_else_help(true)` in the `CliArgs` struct.
    let cli_args = AppArgs::parse();

    let enable_logging = TRACE | cli_args.global_opts.enable_logging;

    call_if_true!(enable_logging, {
        log_debug(format!("cli_args {:?}", cli_args));
    });

    call_if_true!(enable_logging, {
        try_to_set_log_level(log::LevelFilter::Trace).ok();
        log_debug("Start logging...".to_string());
        log_debug(format!("og_size: {:?}", get_size()?).to_string());
    });

    match cli_args.command {
        CLICommand::SelectFromList {
            selection_mode,
            command_to_run_with_selection,
        } => {
            // macos has issues w/ stdin piped in.
            // https://github.com/crossterm-rs/crossterm/issues/396
            if cfg!(target_os = "macos") {
                match (is_stdin_piped(), is_stdout_piped()) {
                    (StdinIsPiped, _) => {
                        show_error_stdin_pipe_does_not_work_on_macos();
                    }
                    (_, StdoutIsPiped) => {
                        show_error_do_not_pipe_stdout(get_bin_name().as_ref());
                    }
                    (StdinIsNotPiped, StdoutIsNotPiped) => {
                        print_help()?;
                    }
                }
            }
            // Linux works fine.
            else {
                match (is_stdin_piped(), is_stdout_piped()) {
                    (StdinIsPiped, StdoutIsNotPiped) => {
                        let tui_height = cli_args.global_opts.tui_height;
                        let tui_width = cli_args.global_opts.tui_width;
                        show_tui(
                            selection_mode,
                            command_to_run_with_selection,
                            tui_height,
                            tui_width,
                            enable_logging,
                        );
                    }
                    (StdinIsPiped, StdoutIsPiped) => {
                        show_error_do_not_pipe_stdout(get_bin_name().as_ref());
                    }
                    (StdinIsNotPiped, StdoutIsPiped) => {
                        show_error_need_to_pipe_stdin(get_bin_name().as_ref());
                        show_error_do_not_pipe_stdout(get_bin_name().as_ref());
                    }
                    (StdinIsNotPiped, StdoutIsNotPiped) => {
                        show_error_need_to_pipe_stdin(get_bin_name().as_ref());
                    }
                }
            }
        }
    }

    call_if_true!(enable_logging, {
        log_debug("Stop logging...".to_string());
    });

    Ok(())
}

fn show_error_stdin_pipe_does_not_work_on_macos() {
    let msg = "Unfortunately at this time macOS `stdin` pipe does not work on macOS.\
                     \nhttps://github.com/crossterm-rs/crossterm/issues/396"
        .blue()
        .to_string();
    println!("{msg}");
}

fn show_error_need_to_pipe_stdin(bin_name: &str) {
    let msg = format!(
        "Please pipe the output of another command into {bin_name}. \
         \n✅ For example: `ls -l | {bin_name} -s single-select`",
    )
    .green()
    .to_string();
    println!("{msg}");
}

fn show_error_do_not_pipe_stdout(bin_name: &str) {
    let msg = format!(
        "Please do *not* pipe the output of {bin_name} to another command. \
         \n❎ For eg, don't do this: `ls -l | {bin_name} -s single-select | cat`",
    )
    .red()
    .to_string();
    println!("{msg}");
}

fn show_tui(
    selection_mode: SelectionMode,
    command_to_run_with_selection: String,
    tui_height: Option<usize>,
    tui_width: Option<usize>,
    enable_logging: bool,
) {
    let lines = stdin().lock().lines().flatten().collect::<Vec<String>>();

    call_if_true!(enable_logging, {
        log_debug(format!("lines: {:?}", lines));
    });

    // Early return, nothing to do. No content found in stdin.
    if lines.is_empty() {
        return;
    }

    // Get display size.
    let max_width_col_count: usize =
        tui_width.unwrap_or(get_size().map(|it| it.col_count).unwrap_or(ch!(80)).into());
    let max_height_row_count: usize = tui_height.unwrap_or(5);

    // Actually get input from the user.
    let selected_items = {
        let it = select_from_list(
            lines,
            max_height_row_count,
            max_width_col_count,
            selection_mode,
        );
        convert_user_input_into_vec_of_strings(it)
    };

    call_if_true!(enable_logging, {
        log_debug(format!("selected_items: {:?}", selected_items));
    });

    for selected_item in selected_items {
        let actual_command_to_run =
            &command_to_run_with_selection.replace(SELECTED_ITEM_SYMBOL, &selected_item);
        execute_command(actual_command_to_run);
    }
}

fn convert_user_input_into_vec_of_strings(
    user_input: Option<Vec<String>>,
) -> Vec<String> {
    match user_input {
        Some(it) => it,
        None => vec![],
    }
}

fn execute_command(cmd_str: &str) {
    // This let binding is required to make the code below work.
    let mut command = if cfg!(target_os = "windows") {
        Command::new("cmd")
    } else {
        Command::new("sh")
    };

    let command = if cfg!(target_os = "windows") {
        command.arg("/C").arg(cmd_str)
    } else {
        command.arg("-c").arg(cmd_str)
    };

    let output = command.output().expect("failed to execute process");
    print!("{}", String::from_utf8_lossy(&output.stdout));
}

/// Programmatically prints out help.
pub fn print_help() -> Result<()> {
    let mut cmd = AppArgs::command();
    cmd.print_help()?;
    Ok(())
}
