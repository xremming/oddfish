use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n},
    character::complete::{multispace0, space0},
    combinator::{map, opt, recognize, value},
    error::ParseError,
    multi::separated_list0,
    number::complete::recognize_float,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    AsChar, IResult, InputTakeAtPosition, Parser,
};
use unicode_ident::{is_xid_continue, is_xid_start};

use crate::Number;

#[inline(always)]
fn padded<F, I, O, E: ParseError<I>>(
    allow_newline: bool,
    parser: F,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: Parser<I, O, E>,
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    let f = if allow_newline { multispace0 } else { space0 };
    delimited(f, parser, f)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ASTNode<'a> {
    Nil,
    Bool(bool),
    Number(Number),
    String(&'a str),
    Ident(&'a str),

    Splat,
    TableList(Vec<ASTNode<'a>>),
    TableDict(Vec<(ASTNode<'a>, ASTNode<'a>)>),

    Expr(Box<ASTNode<'a>>),
    ExprUnary(Op, Box<ASTNode<'a>>),
    ExprBinary(Op, Box<ASTNode<'a>>, Box<ASTNode<'a>>),
}

impl<'a> ASTNode<'a> {
    /// If the node is an ident, return it as a string, otherwise return the node as is.
    fn as_string(self) -> ASTNode<'a> {
        match self {
            ASTNode::Ident(v) => ASTNode::String(v),
            _ => self,
        }
    }
}

fn op_pow(input: &str) -> IResult<&str, Op> {
    value(Op::Pow, tag("^"))(input)
}

fn expr_pow(input: &str) -> IResult<&str, ASTNode> {
    alt((
        map(
            tuple((
                base_value,
                delimited(space0, op_pow, multispace0),
                base_value,
            )),
            |(lhs, op, rhs)| ASTNode::ExprBinary(op, lhs.into(), rhs.into()),
        ),
        base_value,
    ))(input)
}

fn op_mul(input: &str) -> IResult<&str, Op> {
    value(Op::Mul, tag("*"))(input)
}

fn op_div(input: &str) -> IResult<&str, Op> {
    value(Op::Div, tag("/"))(input)
}

fn op_mod(input: &str) -> IResult<&str, Op> {
    value(Op::Mod, tag("%"))(input)
}

fn expr_div_mod(input: &str) -> IResult<&str, ASTNode> {
    alt((
        map(
            tuple((
                expr_pow,
                delimited(space0, alt((op_mul, op_div, op_mod)), multispace0),
                expr_pow,
            )),
            |(lhs, op, rhs)| ASTNode::ExprBinary(op, lhs.into(), rhs.into()),
        ),
        expr_pow,
    ))(input)
}

fn op_add(input: &str) -> IResult<&str, Op> {
    value(Op::Add, tag("+"))(input)
}

fn op_sub(input: &str) -> IResult<&str, Op> {
    value(Op::Sub, tag("-"))(input)
}

fn expr_add_sub(input: &str) -> IResult<&str, ASTNode> {
    alt((
        map(
            tuple((
                expr_div_mod,
                delimited(space0, alt((op_add, op_sub)), multispace0),
                expr_div_mod,
            )),
            |(lhs, op, rhs)| ASTNode::ExprBinary(op, lhs.into(), rhs.into()),
        ),
        expr_div_mod,
    ))(input)
}

fn expr_group(input: &str) -> IResult<&str, ASTNode> {
    map(
        alt((
            delimited(
                terminated(tag("("), multispace0),
                expr_add_sub,
                preceded(multispace0, tag(")")),
            ),
            expr_add_sub,
        )),
        |v| ASTNode::Expr(v.into()),
    )(input)
}

fn expr(input: &str) -> IResult<&str, ASTNode> {
    alt((expr_group, table, primitive))(input)
}

fn ident(input: &str) -> IResult<&str, ASTNode> {
    map(
        recognize(tuple((
            take_while_m_n(1, 1, |c| is_xid_start(c) || c == '_'),
            take_while(is_xid_continue),
        ))),
        |v| ASTNode::Ident(v),
    )(input)
}

fn nil(input: &str) -> IResult<&str, ASTNode> {
    value(ASTNode::Nil, tag("nil"))(input)
}

fn bool_true(input: &str) -> IResult<&str, ASTNode> {
    value(ASTNode::Bool(true), tag("true"))(input)
}

fn bool_false(input: &str) -> IResult<&str, ASTNode> {
    value(ASTNode::Bool(false), tag("false"))(input)
}

fn bool(input: &str) -> IResult<&str, ASTNode> {
    alt((bool_false, bool_true))(input)
}

fn number(input: &str) -> IResult<&str, ASTNode> {
    map(recognize_float, |v| ASTNode::Number(Number::parse(v)))(input)
}

fn primitive(input: &str) -> IResult<&str, ASTNode> {
    alt((nil, bool, number, ident))(input)
}

fn list_element(input: &str) -> IResult<&str, ASTNode> {
    primitive(input)
}

fn list(input: &str) -> IResult<&str, ASTNode> {
    map(
        delimited(
            terminated(tag("["), multispace0),
            terminated(
                separated_list0(tag(","), padded(true, list_element)),
                opt(tag(",")),
            ),
            preceded(multispace0, tag("]")),
        ),
        |v| ASTNode::TableList(v),
    )(input)
}

fn dict_key_value(input: &str) -> IResult<&str, (ASTNode, ASTNode)> {
    map(
        separated_pair(primitive, padded(true, tag("=")), primitive),
        |(a, b)| (a.as_string(), b),
    )(input)
}

fn dict_splat(input: &str) -> IResult<&str, (ASTNode, ASTNode)> {
    // TODO: use expr instead of ident?
    map(preceded(tag("..."), preceded(multispace0, ident)), |v| {
        (ASTNode::Splat, v)
    })(input)
}

fn dict_key_shorthand(input: &str) -> IResult<&str, (ASTNode, ASTNode)> {
    map(ident, |v| (v.clone().as_string(), v))(input)
}

fn dict_element(input: &str) -> IResult<&str, (ASTNode, ASTNode)> {
    alt((dict_key_value, dict_splat, dict_key_shorthand))(input)
}

fn dict(input: &str) -> IResult<&str, ASTNode> {
    map(
        delimited(
            terminated(tag("{"), multispace0),
            terminated(
                separated_list0(tag(","), padded(true, dict_element)),
                opt(tag(",")),
            ),
            preceded(multispace0, tag("}")),
        ),
        |v| ASTNode::TableDict(v),
    )(input)
}

fn table(input: &str) -> IResult<&str, ASTNode> {
    alt((list, dict))(input)
}

fn base_value(input: &str) -> IResult<&str, ASTNode> {
    alt((table, primitive))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use ASTNode::*;

    macro_rules! test_parser {
        ($name:ident => $parser:ident
            ok [$($ok_input:expr => $ok_expected:expr),* $(,)?]
            partial [$($partial_input:expr => $partial_expected:expr),* $(,)?]
            fail [$($fail_input:expr),* $(,)?]
        ) => {
            #[test]
            fn $name() {
                $(
                    assert_eq!($parser($ok_input), Ok(("", $ok_expected)));
                )*
                $(
                    assert_eq!($parser($partial_input), Ok($partial_expected));
                )*
                $(
                    assert!($parser($fail_input).is_err());
                )*
            }
        };
    }

    test_parser!(
        test_ident => ident
        ok [
            "_"          => Ident("_"),
            "abba"       => Ident("abba"),
            "abba_baab"  => Ident("abba_baab"),
            "_abba"      => Ident("_abba"),
            "_abba_baab" => Ident("_abba_baab"),
            "ääkköset"   => Ident("ääkköset"),
        ]
        partial [
            "_   "     => ("   ", Ident("_")),
            "abba   "  => ("   ", Ident("abba")),
            "_abba   " => ("   ", Ident("_abba")),
        ]
        fail [
            "",
            " ",
            "1",
            ".",
            "1abc",
            "   abc",
            "   _abc",
            "   _abc   ",
        ]
    );

    macro_rules! num {
        ($v:expr) => {
            ASTNode::Number($v.into())
        };
    }

    test_parser!(
        test_list => list
        ok [
            "[]"    => TableList(vec![]),
            "[ ]"   => TableList(vec![]),
            "[ ,]"  => TableList(vec![]),
            "[, ]"  => TableList(vec![]),
            "[ , ]" => TableList(vec![]),

            "[a]"    => TableList(vec![Ident("a")]),
            "[ a]"   => TableList(vec![Ident("a")]),
            "[a ]"   => TableList(vec![Ident("a")]),
            "[ a ]"  => TableList(vec![Ident("a")]),
            "[a,]"   => TableList(vec![Ident("a")]),
            "[ a,]"  => TableList(vec![Ident("a")]),
            "[a, ]"  => TableList(vec![Ident("a")]),
            "[ a, ]" => TableList(vec![Ident("a")]),

            "[a,b,c]"        => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[ a, b, c]"     => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[a ,b ,c ]"     => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[ a , b , c ]"  => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[a,b,c,]"       => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[ a, b, c,]"    => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[a ,b ,c ,]"    => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),
            "[ a , b , c ,]" => TableList(vec![Ident("a"), Ident("b"), Ident("c")]),

            "[1,2,3]"         => TableList(vec![num!(1), num!(2), num!(3)]),
            "[ 1, 2, 3]"      => TableList(vec![num!(1), num!(2), num!(3)]),
            "[1 ,2 ,3 ]"      => TableList(vec![num!(1), num!(2), num!(3)]),
            "[ 1 , 2 , 3 ]"   => TableList(vec![num!(1), num!(2), num!(3)]),
            "[1,2,3,]"        => TableList(vec![num!(1), num!(2), num!(3)]),
            "[ 1, 2, 3,]"     => TableList(vec![num!(1), num!(2), num!(3)]),
            "[1 ,2 ,3 ,]"     => TableList(vec![num!(1), num!(2), num!(3)]),
            "[ 1 , 2 , 3 , ]" => TableList(vec![num!(1), num!(2), num!(3)]),

            "[nil,true,false]"         => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[ nil, true, false]"      => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[nil ,true ,false ]"      => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[ nil , true , false ]"   => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[nil,true,false,]"        => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[ nil, true, false, ]"    => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[nil ,true ,false , ]"    => TableList(vec![Nil, Bool(true), Bool(false)]),
            "[ nil , true , false , ]" => TableList(vec![Nil, Bool(true), Bool(false)]),
        ]
        partial []
        fail [
            "",
            "[",
            "[,",
            "]",
            ",]",
            "[,,]",
            "[ ,,]",
            "[,, ]",
            "[ ,, ]",
            "[, ,]",
            "[ , ,]",
            "[, , ]",
            "[ , , ]",
        ]
    );

    test_parser!(
        test_dict => dict
        ok [
            "{}"    => TableDict(vec![]),
            "{ }"   => TableDict(vec![]),
            "{,}"   => TableDict(vec![]),
            "{ ,}"  => TableDict(vec![]),
            "{, }"  => TableDict(vec![]),
            "{ , }" => TableDict(vec![]),

            "{a=1}"      => TableDict(vec![(String("a"), num!(1))]),
            "{ a=1}"     => TableDict(vec![(String("a"), num!(1))]),
            "{a=1 }"     => TableDict(vec![(String("a"), num!(1))]),
            "{ a=1 }"    => TableDict(vec![(String("a"), num!(1))]),
            "{a =1}"     => TableDict(vec![(String("a"), num!(1))]),
            "{ a =1}"    => TableDict(vec![(String("a"), num!(1))]),
            "{a =1 }"    => TableDict(vec![(String("a"), num!(1))]),
            "{ a =1 }"   => TableDict(vec![(String("a"), num!(1))]),
            "{a= 1}"     => TableDict(vec![(String("a"), num!(1))]),
            "{ a= 1}"    => TableDict(vec![(String("a"), num!(1))]),
            "{a= 1 }"    => TableDict(vec![(String("a"), num!(1))]),
            "{ a= 1 }"   => TableDict(vec![(String("a"), num!(1))]),
            "{a = 1}"    => TableDict(vec![(String("a"), num!(1))]),
            "{ a = 1}"   => TableDict(vec![(String("a"), num!(1))]),
            "{a = 1 }"   => TableDict(vec![(String("a"), num!(1))]),
            "{ a = 1 }"  => TableDict(vec![(String("a"), num!(1))]),
            "{a=1,}"     => TableDict(vec![(String("a"), num!(1))]),
            "{ a=1,}"    => TableDict(vec![(String("a"), num!(1))]),
            "{a=1, }"    => TableDict(vec![(String("a"), num!(1))]),
            "{ a=1, }"   => TableDict(vec![(String("a"), num!(1))]),
            "{a =1,}"    => TableDict(vec![(String("a"), num!(1))]),
            "{ a =1,}"   => TableDict(vec![(String("a"), num!(1))]),
            "{a =1, }"   => TableDict(vec![(String("a"), num!(1))]),
            "{ a =1, }"  => TableDict(vec![(String("a"), num!(1))]),
            "{a= 1,}"    => TableDict(vec![(String("a"), num!(1))]),
            "{ a= 1,}"   => TableDict(vec![(String("a"), num!(1))]),
            "{a= 1, }"   => TableDict(vec![(String("a"), num!(1))]),
            "{ a= 1, }"  => TableDict(vec![(String("a"), num!(1))]),
            "{a = 1,}"   => TableDict(vec![(String("a"), num!(1))]),
            "{ a = 1,}"  => TableDict(vec![(String("a"), num!(1))]),
            "{a = 1, }"  => TableDict(vec![(String("a"), num!(1))]),
            "{ a = 1, }" => TableDict(vec![(String("a"), num!(1))]),
        ]
        partial []
        fail []
    );
}
