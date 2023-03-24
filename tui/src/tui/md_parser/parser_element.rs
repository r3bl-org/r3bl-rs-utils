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

use constants::*;
use nom::{branch::*,
          bytes::complete::*,
          character::complete::*,
          combinator::*,
          multi::*,
          sequence::*,
          IResult};

use crate::*;

#[rustfmt::skip]
pub fn parse_element_bold_italic(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(/* start */ tag(BITALIC_1), /* output */ is_not(BITALIC_1), /* end */ tag(BITALIC_1)),
        delimited(/* start */ tag(BITALIC_2), /* output */ is_not(BITALIC_2), /* end */ tag(BITALIC_2)),
    ))(input)
}

#[rustfmt::skip]
pub fn parse_element_bold(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(/* start */ tag(BOLD_1), /* output */ is_not(BOLD_1), /* end */ tag(BOLD_1)),
        delimited(/* start */ tag(BOLD_2), /* output */ is_not(BOLD_2), /* end */ tag(BOLD_2)),
    ))(input)
}

#[rustfmt::skip]
pub fn parse_element_italic(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(/* start */ tag(ITALIC_1), /* output */ is_not(ITALIC_1), /* end */ tag(ITALIC_1)),
        delimited(/* start */ tag(ITALIC_2), /* output */ is_not(ITALIC_2), /* end */ tag(ITALIC_2)),
    ))(input)
}

#[rustfmt::skip]
pub fn parse_element_code(input: &str) -> IResult<&str, &str> {
    delimited(/* start */ tag(BACK_TICK), /* output */ is_not(BACK_TICK), /* end */ tag(BACK_TICK))(input)
}

#[rustfmt::skip]
pub fn parse_element_link(input: &str) -> IResult<&str, HyperlinkData> {
    let (input, output) = pair(
        delimited(/* start */ tag(LEFT_BRACKET), /* output */ is_not(RIGHT_BRACKET), /* end */ tag(RIGHT_BRACKET)),
        delimited(/* start */ tag(LEFT_PARENTHESIS), /* output */ is_not(RIGHT_PARENTHESIS), /* end */ tag(RIGHT_PARENTHESIS)),
    )(input)?;
    Ok((input, HyperlinkData::from(output)))
}

#[rustfmt::skip]
pub fn parse_element_checkbox(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag(CHECKED), |_| true),
        map(tag(UNCHECKED), |_| false),
    ))(input)
}

#[rustfmt::skip]
pub fn parse_element_image(input: &str) -> IResult<&str, HyperlinkData> {
    let (input, output) =pair(
        delimited(/* start */ tag(LEFT_IMAGE), /* output */ is_not(RIGHT_IMAGE), /* end */ tag(RIGHT_IMAGE)),
        delimited(/* start */ tag(LEFT_PARENTHESIS), /* output */ is_not(RIGHT_PARENTHESIS), /* end */ tag(RIGHT_PARENTHESIS)),
    )(input)?;
    Ok((input, HyperlinkData::from(output)))
}

/// There must be at least one match. We want to match many things that are not any of our
/// special tags, but since we have no tools available to match and consume in the negative case
/// (without regex) we need to match against our (start) tags, then consume one char; we repeat
/// this until we run into one of our special characters (start tags) then we return this slice.
#[rustfmt::skip]
pub fn parse_element_plaintext(input: &str) -> IResult<&str, &str> {
    recognize(
        many1(
            preceded(
                /* prefix - discarded */
                not(
                    /* starts with special characters */
                    alt((
                        tag(BITALIC_1),
                        tag(BITALIC_2),
                        tag(BOLD_1),
                        tag(BOLD_2),
                        tag(ITALIC_1),
                        tag(ITALIC_2),
                        tag(BACK_TICK),
                        tag(LEFT_BRACKET),
                        tag(LEFT_IMAGE),
                        tag(NEW_LINE),
                    ))
                ),
                /* output - keep char */
                anychar,
            )
        )
    )(input)
}

/// Parse a single chunk of markdown text [MdLineFragment] in a single line.
#[rustfmt::skip]
pub fn parse_element_markdown_inline(input: &str) -> IResult<&str, MdLineFragment> {
    alt((
        map(parse_element_italic,       MdLineFragment::Italic),
        map(parse_element_bold,         MdLineFragment::Bold),
        map(parse_element_bold_italic,  MdLineFragment::BoldItalic),
        map(parse_element_code,         MdLineFragment::InlineCode),
        map(parse_element_image,        MdLineFragment::Image),
        map(parse_element_link,         MdLineFragment::Link),
        map(parse_element_checkbox,     MdLineFragment::Checkbox),
        map(parse_element_plaintext,    MdLineFragment::Plain),
    ))(input)
}

#[cfg(test)]
mod tests {
    use nom::{error::{Error, ErrorKind},
              Err as NomErr};
    use r3bl_rs_utils_core::assert_eq2;

    use super::*;

    #[test]
    fn test_parse_element_italic() {
        assert_eq2!(
            parse_element_italic("*here is italic*"),
            Ok(("", "here is italic"))
        );

        assert_eq2!(
            parse_element_italic("_here is italic_"),
            Ok(("", "here is italic"))
        );

        assert_eq2!(
            parse_element_italic("*here is italic"),
            Err(NomErr::Error(Error {
                input: "*here is italic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_italic("here is italic*"),
            Err(NomErr::Error(Error {
                input: "here is italic*",
                code: ErrorKind::Tag,
            }))
        );

        assert_eq2!(
            parse_element_italic("here is italic"),
            Err(NomErr::Error(Error {
                input: "here is italic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_italic("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_italic("**"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_italic(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_italic("**we are doing bold**"),
            Err(NomErr::Error(Error {
                input: "**we are doing bold**",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_element_bold_italic() {
        assert_eq2!(
            parse_element_bold_italic("***here is bitalic***"),
            Ok(("", "here is bitalic"))
        );

        assert_eq2!(
            parse_element_bold("***here is bitalic"),
            Err(NomErr::Error(Error {
                input: "***here is bitalic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("here is bitalic***"),
            Err(NomErr::Error(Error {
                input: "here is bitalic***",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold_italic("___here is bitalic___"),
            Ok(("", "here is bitalic"))
        );

        assert_eq2!(
            parse_element_bold_italic("___here is bitalic"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold_italic("here is bitalic___"),
            Err(NomErr::Error(Error {
                input: "here is bitalic___",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_element_bold() {
        assert_eq2!(
            parse_element_bold("**here is bold**"),
            Ok(("", "here is bold"))
        );

        assert_eq2!(
            parse_element_bold("__here is bold__"),
            Ok(("", "here is bold"))
        );

        assert_eq2!(
            parse_element_bold("**here is bold"),
            Err(NomErr::Error(Error {
                input: "**here is bold",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("here is bold**"),
            Err(NomErr::Error(Error {
                input: "here is bold**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("here is bold"),
            Err(NomErr::Error(Error {
                input: "here is bold",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("****"),
            Err(NomErr::Error(Error {
                input: "****",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("**"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq2!(
            parse_element_bold("*this is italic*"),
            Err(NomErr::Error(Error {
                input: "*this is italic*",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_element_code() {
        assert_eq2!(
            parse_element_code("`here is code"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq2!(
            parse_element_code("here is code`"),
            Err(NomErr::Error(Error {
                input: "here is code`",
                code: ErrorKind::Tag
            }))
        );
        assert_eq2!(
            parse_element_code("``"),
            Err(NomErr::Error(Error {
                input: "`",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq2!(
            parse_element_code("`"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq2!(
            parse_element_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_element_link() {
        assert_eq2!(
            parse_element_link("[title](https://www.example.com)"),
            Ok(("", HyperlinkData::new("title", "https://www.example.com")))
        );
        assert_eq2!(
            parse_element_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_element_image() {
        assert_eq2!(
            parse_element_image("![alt text](image.jpg)"),
            Ok(("", HyperlinkData::new("alt text", "image.jpg")))
        );
        assert_eq2!(
            parse_element_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_element_plaintext() {
        assert_eq2!(
            parse_element_plaintext("1234567890"),
            Ok(("", "1234567890"))
        );
        assert_eq2!(
            parse_element_plaintext("oh my gosh!"),
            Ok(("", "oh my gosh!"))
        );
        assert_eq2!(
            parse_element_plaintext("oh my gosh!["),
            Ok(("![", "oh my gosh"))
        );
        assert_eq2!(
            parse_element_plaintext("oh my gosh!*"),
            Ok(("*", "oh my gosh!"))
        );
        assert_eq2!(
            parse_element_plaintext("*bold baby bold*"),
            Err(NomErr::Error(Error {
                input: "*bold baby bold*",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("[link baby](and then somewhat)"),
            Err(NomErr::Error(Error {
                input: "[link baby](and then somewhat)",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("`codeblock for bums`"),
            Err(NomErr::Error(Error {
                input: "`codeblock for bums`",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("![ but wait theres more](jk)"),
            Err(NomErr::Error(Error {
                input: "![ but wait theres more](jk)",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("here is plaintext"),
            Ok(("", "here is plaintext"))
        );
        assert_eq2!(
            parse_element_plaintext("here is plaintext!"),
            Ok(("", "here is plaintext!"))
        );
        assert_eq2!(
            parse_element_plaintext("here is plaintext![image starting"),
            Ok(("![image starting", "here is plaintext"))
        );
        assert_eq2!(
            parse_element_plaintext("here is plaintext\n"),
            Ok(("\n", "here is plaintext"))
        );
        assert_eq2!(
            parse_element_plaintext("*here is italic*"),
            Err(NomErr::Error(Error {
                input: "*here is italic*",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("**here is bold**"),
            Err(NomErr::Error(Error {
                input: "**here is bold**",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("`here is code`"),
            Err(NomErr::Error(Error {
                input: "`here is code`",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("[title](https://www.example.com)"),
            Err(NomErr::Error(Error {
                input: "[title](https://www.example.com)",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext("![alt text](image.jpg)"),
            Err(NomErr::Error(Error {
                input: "![alt text](image.jpg)",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_plaintext(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_element_markdown_inline() {
        assert_eq2!(
            parse_element_markdown_inline("*here is italic*"),
            Ok(("", MdLineFragment::Italic("here is italic")))
        );
        assert_eq2!(
            parse_element_markdown_inline("**here is bold**"),
            Ok(("", MdLineFragment::Bold("here is bold")))
        );
        assert_eq2!(
            parse_element_markdown_inline("`here is code`"),
            Ok(("", MdLineFragment::InlineCode("here is code")))
        );
        assert_eq2!(
            parse_element_markdown_inline("[title](https://www.example.com)"),
            Ok((
                "",
                (MdLineFragment::Link(HyperlinkData::new("title", "https://www.example.com")))
            ))
        );
        assert_eq2!(
            parse_element_markdown_inline("![alt text](image.jpg)"),
            Ok((
                "",
                (MdLineFragment::Image(HyperlinkData::new("alt text", "image.jpg")))
            ))
        );
        assert_eq2!(
            parse_element_markdown_inline("here is plaintext!"),
            Ok(("", MdLineFragment::Plain("here is plaintext!")))
        );
        assert_eq2!(
            parse_element_markdown_inline("here is some plaintext *but what if we italicize?"),
            Ok((
                "*but what if we italicize?",
                MdLineFragment::Plain("here is some plaintext ")
            ))
        );
        assert_eq2!(
            parse_element_markdown_inline("here is some plaintext \n*but what if we italicize?"),
            Ok((
                "\n*but what if we italicize?",
                MdLineFragment::Plain("here is some plaintext ")
            ))
        );
        assert_eq2!(
            parse_element_markdown_inline("\n"),
            Err(NomErr::Error(Error {
                input: "\n",
                code: ErrorKind::Not
            }))
        );
        assert_eq2!(
            parse_element_markdown_inline(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }
}