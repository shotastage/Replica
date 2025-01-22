use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
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

fn keyword(input: &str) -> IResult<&str, Token> {
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
    ))(input)
}

fn operator(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("->"), |_| Token::Arrow),
        map(char('{'), |_| Token::LBrace),
        map(char('}'), |_| Token::RBrace),
        map(char('('), |_| Token::LParen),
        map(char(')'), |_| Token::RParen),
        map(char(':'), |_| Token::Colon),
        map(char(','), |_| Token::Comma),
        map(char('='), |_| Token::Equals),
    ))(input)
}

fn identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| Token::Identifier(s.to_string()),
    )(input)
}

fn string_literal(input: &str) -> IResult<&str, Token> {
    map(
        preceded(
            char('"'),
            terminated(take_while1(|c| c != '"'), char('"')),
        ),
        |s: &str| Token::StringLiteral(s.to_string()),
    )(input)
}

fn number_literal(input: &str) -> IResult<&str, Token> {
    map(
        recognize(many0(alt((alphanumeric1, tag("."))))),
        |s: &str| Token::NumberLiteral(s.to_string()),
    )(input)
}

fn token(input: &str) -> IResult<&str, Token> {
    alt((
        keyword,
        operator,
        identifier,
        string_literal,
        number_literal,
    ))(input)
}

pub fn lex(input: &str) -> IResult<&str, Vec<Token>> {
    many0(terminated(token, multispace0))(input)
}
