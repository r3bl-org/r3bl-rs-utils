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

// Attach.
pub mod color_text_default_styles;
pub mod console_log_impl;
pub mod proper_logging_impl;
pub mod simple_file_logging_impl;

// Re-export.
pub use color_text_default_styles::*;
pub use console_log_impl::*;
pub use proper_logging_impl::*;
pub use simple_file_logging_impl::*;