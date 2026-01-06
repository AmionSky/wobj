// use winnow::ascii::{
//     float, line_ending, multispace0, multispace1, space0, space1, till_line_ending,
// };
// use winnow::combinator::{delimited, dispatch, fail, opt, preceded, repeat, seq, terminated};
// use winnow::error::{ParserError, StrContext, StrContextValue};
// use winnow::prelude::*;
// use winnow::stream::{Accumulate, AsChar};
// use winnow::token::{take, take_till, take_while};
// use winnow::{BStr, Result};

use wobj::Obj;


fn main() {
    Obj::parse("/home/csanyi/Projects/bevy_obj/assets/cube.obj");
}

#[cfg(test)]
mod tests {
    use super::*;
}
