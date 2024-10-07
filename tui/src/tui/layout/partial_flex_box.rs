/*
 *   Copyright (c) 2024 R3BL LLC
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

use r3bl_core::{Position, Size, TuiStyle};
use serde::{Deserialize, Serialize};

use super::{FlexBox, FlexBoxId};
use crate::format_option;

/// Holds a subset of the fields in [FlexBox] that are required by the editor and dialog
/// engines.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialFlexBox {
    pub id: FlexBoxId,
    pub style_adjusted_origin_pos: Position,
    pub style_adjusted_bounds_size: Size,
    pub maybe_computed_style: Option<TuiStyle>,
}

impl PartialFlexBox {
    pub fn get_computed_style(&self) -> Option<TuiStyle> { self.maybe_computed_style }

    pub fn get_style_adjusted_position_and_size(&self) -> (Position, Size) {
        (
            self.style_adjusted_origin_pos,
            self.style_adjusted_bounds_size,
        )
    }
}

impl Debug for PartialFlexBox {
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

impl From<PartialFlexBox> for FlexBox {
    fn from(engine_box: PartialFlexBox) -> Self {
        Self {
            id: engine_box.id,
            style_adjusted_origin_pos: engine_box.style_adjusted_origin_pos,
            style_adjusted_bounds_size: engine_box.style_adjusted_bounds_size,
            maybe_computed_style: engine_box.get_computed_style(),
            ..Default::default()
        }
    }
}

impl From<FlexBox> for PartialFlexBox {
    fn from(flex_box: FlexBox) -> Self { PartialFlexBox::from(&flex_box) }
}

impl From<&FlexBox> for PartialFlexBox {
    fn from(flex_box: &FlexBox) -> Self {
        Self {
            id: flex_box.id,
            style_adjusted_origin_pos: flex_box.style_adjusted_origin_pos,
            style_adjusted_bounds_size: flex_box.style_adjusted_bounds_size,
            maybe_computed_style: flex_box.get_computed_style(),
        }
    }
}

#[cfg(test)]
mod tests {
    use r3bl_core::{position, size};

    use super::*;

    #[test]
    fn test_partial_flex_box_default() {
        let partial_flex_box = PartialFlexBox::default();
        assert_eq!(partial_flex_box.id, FlexBoxId::default());
        assert_eq!(
            partial_flex_box.style_adjusted_origin_pos,
            Position::default()
        );
        assert_eq!(partial_flex_box.style_adjusted_bounds_size, Size::default());
        assert_eq!(partial_flex_box.maybe_computed_style, None);
    }

    #[test]
    fn test_partial_flex_box_get_computed_style() {
        let style = TuiStyle::default();
        let partial_flex_box = PartialFlexBox {
            maybe_computed_style: Some(style),
            ..Default::default()
        };
        assert_eq!(partial_flex_box.get_computed_style(), Some(style));
    }

    #[test]
    fn test_partial_flex_box_get_style_adjusted_position_and_size() {
        let position = position!(col_index: 1, row_index: 2);
        let size = size!(col_count: 3, row_count: 4);
        let partial_flex_box = PartialFlexBox {
            style_adjusted_origin_pos: position,
            style_adjusted_bounds_size: size,
            ..Default::default()
        };
        assert_eq!(
            partial_flex_box.get_style_adjusted_position_and_size(),
            (position, size)
        );
    }

    #[test]
    fn test_partial_flex_box_debug() {
        let partial_flex_box = PartialFlexBox::default();
        let debug_str = format!("{:?}", partial_flex_box);
        assert!(debug_str.contains("FlexBox"));
        assert!(debug_str.contains("id"));
        assert!(debug_str.contains("style_adjusted_origin_pos"));
        assert!(debug_str.contains("style_adjusted_bounds_size"));
        assert!(debug_str.contains("maybe_computed_style"));
    }

    #[test]
    fn test_partial_flex_box_from_flex_box() {
        let flex_box = FlexBox::default();
        let partial_flex_box: PartialFlexBox = flex_box.into();
        assert_eq!(partial_flex_box.id, FlexBoxId::default());
        assert_eq!(
            partial_flex_box.style_adjusted_origin_pos,
            Position::default()
        );
        assert_eq!(partial_flex_box.style_adjusted_bounds_size, Size::default());
        assert_eq!(partial_flex_box.maybe_computed_style, None);
    }

    #[test]
    fn test_flex_box_from_partial_flex_box() {
        let partial_flex_box = PartialFlexBox::default();
        let flex_box: FlexBox = partial_flex_box.into();
        assert_eq!(flex_box.id, FlexBoxId::default());
        assert_eq!(flex_box.style_adjusted_origin_pos, Position::default());
        assert_eq!(flex_box.style_adjusted_bounds_size, Size::default());
        assert_eq!(flex_box.maybe_computed_style, None);
    }
}