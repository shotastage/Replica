use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1},
    combinator::{map, recognize},
    multi::many0,
    sequence::{pair, preceded, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Actor,
    SingleActor,
    Var,
    Let,
    Func,
    Async,
    Sequential,
    Immediate,
    Move,
    Copy,
    Shared,
    Init,
    Arrow,
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Comma,
    Equals,
}

pub fn lex(input: &str) -> IResult<&str, Vec<Token>> {
    many0(terminated(
        alt((
            map(tag("actor"), |_| Token::Actor),
            map(tag("single actor"), |_| Token::SingleActor),
            map(tag("var"), |_| Token::Var),
            map(tag("let"), |_| Token::Let),
            map(tag("func"), |_| Token::Func),
            map(tag("async"), |_| Token::Async),
            map(tag("sequential"), |_| Token::Sequential),
            map(tag("immediate"), |_| Token::Immediate),
            map(tag("move"), |_| Token::Move),
            map(tag("copy"), |_| Token::Copy),
            map(tag("shared"), |_| Token::Shared),
            map(tag("init"), |_| Token::Init),
            map(tag("->"), |_| Token::Arrow),
            map(parse_identifier, |s| Token::Identifier(s.to_string())),
            map(parse_string_literal, |s| {
                Token::StringLiteral(s.to_string())
            }),
            map(parse_number_literal, |s| {
                Token::NumberLiteral(s.to_string())
            }),
            map(char('{'), |_| Token::LBrace),
            map(char('}'), |_| Token::RBrace),
            map(char('('), |_| Token::LParen),
            map(char(')'), |_| Token::RParen),
            map(char(':'), |_| Token::Colon),
            map(char(','), |_| Token::Comma),
            map(char('='), |_| Token::Equals),
        )),
        multispace0,
    ))(input)
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, &str> {
    preceded(char('"'), terminated(take_while1(|c| c != '"'), char('"')))(input)
}

fn parse_number_literal(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((alphanumeric1, tag(".")))))(input)
}
