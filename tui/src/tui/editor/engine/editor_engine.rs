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

use std::fmt::Debug;

use r3bl_rs_utils_core::*;
use serde::*;
use syntect::{highlighting::Theme, parsing::SyntaxSet};

use crate::*;

// ┏━━━━━━━━━━━━━━━━━━━━━┓
// ┃ EditorEngine struct ┃
// ┛                     ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Holds data related to rendering in between render calls. This is not stored in the
/// [EditorBuffer] struct, which lives in the [r3bl_redux::Store]. The store provides the underlying
/// document or buffer struct that holds the actual document.
///
/// In order to change the document, you can use the
/// [apply_event](EditorEngineRenderApi::apply_event) method which takes [InputEvent] and tries to
/// convert it to an [EditorEvent] and then execute them against this buffer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorEngine {
    /// Set by [render](EditorEngineRenderApi::render_engine).
    pub current_box: EditorEngineFlexBox,
    pub config_options: EditorEngineConfigOptions,
    /// Syntax highlighting support. This is a very heavy object to create, re-use it.
    pub syntax_set: SyntaxSet,
    /// Syntax highlighting support. This is a very heavy object to create, re-use it.
    pub theme: Theme,
}

mod layout_struct_helper {
    use super::*;

    /// Holds a subset of the fields in [FlexBox] that are required by the editor engine.
    #[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct EditorEngineFlexBox {
        pub id: FlexBoxId,
        pub style_adjusted_origin_pos: Position,
        pub style_adjusted_bounds_size: Size,
        pub maybe_computed_style: Option<Style>,
    }

    impl EditorEngineFlexBox {
        pub fn get_computed_style(&self) -> Option<Style> { self.maybe_computed_style.clone() }
    }

    impl Debug for EditorEngineFlexBox {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("FlexBox")
                .field("id", &self.id)
                .field("style_adjusted_origin_pos", &self.style_adjusted_origin_pos)
                .field(
                    "style_adjusted_bounds_size",
                    &self.style_adjusted_bounds_size,
                )
                .field(
                    "maybe_computed_style",
                    format_option!(&self.maybe_computed_style),
                )
                .finish()
        }
    }

    impl From<EditorEngineFlexBox> for FlexBox {
        fn from(engine_box: EditorEngineFlexBox) -> Self {
            Self {
                id: engine_box.id,
                style_adjusted_origin_pos: engine_box.style_adjusted_origin_pos,
                style_adjusted_bounds_size: engine_box.style_adjusted_bounds_size,
                maybe_computed_style: engine_box.maybe_computed_style,
                ..Default::default()
            }
        }
    }

    impl From<&FlexBox> for EditorEngineFlexBox {
        fn from(flex_box: &FlexBox) -> Self {
            Self {
                id: flex_box.id,
                style_adjusted_origin_pos: flex_box.style_adjusted_origin_pos,
                style_adjusted_bounds_size: flex_box.style_adjusted_bounds_size,
                maybe_computed_style: flex_box.get_computed_style(),
            }
        }
    }
}
pub use layout_struct_helper::*;

mod constructor {
    use super::*;

    impl Default for EditorEngine {
        fn default() -> Self { EditorEngine::new(Default::default()) }
    }

    impl EditorEngine {
        /// Syntax highlighting support - [SyntaxSet] and [Theme] are a very expensive objects to
        /// create, so re-use them.
        pub fn new(config_options: EditorEngineConfigOptions) -> Self {
            Self {
                current_box: Default::default(),
                config_options,
                syntax_set: SyntaxSet::load_defaults_newlines(),
                theme: try_load_r3bl_theme().unwrap_or_else(|_| load_default_theme()),
            }
        }
    }
}

impl EditorEngine {
    pub fn viewport_width(&self) -> ChUnit { self.current_box.style_adjusted_bounds_size.col_count }

    pub fn viewport_height(&self) -> ChUnit {
        self.current_box.style_adjusted_bounds_size.row_count
    }
}

// ┏━━━━━━━━━━━━━━━━┓
// ┃ Config options ┃
// ┛                ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEngineConfigOptions {
    pub multiline: bool,
    pub syntax_highlight: bool,
}

impl Default for EditorEngineConfigOptions {
    fn default() -> Self {
        Self {
            multiline: true,
            syntax_highlight: true,
        }
    }
}