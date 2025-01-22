use crate::ast::*;
use crate::lexer::Token;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected:?}, found {found:?}")]
    UnexpectedToken {
        expected: &'static str,
        found: Token,
    },
    #[error("Unexpected end of input")]
    UnexpectedEOF,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse_actor(&mut self) -> Result<Actor, ParseError> {
        let actor_type = match self.peek() {
            Some(Token::Actor) => {
                self.advance();
                ActorType::Distributed
            }
            Some(Token::SingleActor) => {
                self.advance();
                ActorType::Single
            }
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "actor or single actor",
                    found: token.clone(),
                })
            }
            None => return Err(ParseError::UnexpectedEOF),
        };

        let name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier",
                    found: token,
                })
            }
            None => return Err(ParseError::UnexpectedEOF),
        };

        self.expect(Token::LBrace)?;

        let mut methods = Vec::new();
        let mut fields = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Var | Token::Let => {
                    fields.push(self.parse_field()?);
                }
                Token::Func | Token::Immediate => {
                    methods.push(self.parse_method()?);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "field or method declaration",
                        found: token.clone(),
                    })
                }
            }
        }

        Ok(Actor {
            name,
            actor_type,
            methods,
            fields,
        })
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let is_mutable = match self.advance() {
            Some(Token::Var) => true,
            Some(Token::Let) => false,
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "var or let",
                    found: token,
                })
            }
            None => return Err(ParseError::UnexpectedEOF),
        };

        let name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier",
                    found: token,
                })
            }
            None => return Err(ParseError::UnexpectedEOF),
        };

        self.expect(Token::Colon)?;

        let field_type = self.parse_type()?;
        let mut ownership = OwnershipType::Owned;

        if let Some(Token::Move) = self.peek() {
            self.advance();
            ownership = OwnershipType::Moved;
        }

        Ok(Field {
            name,
            field_type,
            is_mutable,
            ownership,
        })
    }

    // Helper methods
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current);
        self.current += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.advance() {
            Some(token) if token == &expected => Ok(()),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "expected token",
                found: token.clone(),
            }),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}
