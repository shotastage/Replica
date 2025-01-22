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
        Parser {
            tokens,
            current: 0,
        }
    }

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
                    found: token.clone(),
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

    fn parse_method(&mut self) -> Result<Method, ParseError> {
        let is_immediate = if let Some(Token::Immediate) = self.peek() {
            self.advance();
            true
        } else {
            false
        };

        self.expect(Token::Func)?;

        let name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier",
                    found: token.clone(),
                })
            }
            None => return Err(ParseError::UnexpectedEOF),
        };

        self.expect(Token::LParen)?;
        let params = self.parse_parameters()?;
        self.expect(Token::RParen)?;

        let return_type = if let Some(Token::Arrow) = self.peek() {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(Method {
            name,
            is_async: true,
            is_sequential: false,
            is_immediate,
            params,
            return_type,
        })
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let is_mutable = match self.advance() {
            Some(Token::Var) => true,
            Some(Token::Let) => false,
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "var or let",
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
                    found: token.clone(),
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

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.advance() {
            Some(Token::Identifier(type_name)) => {
                match type_name.as_str() {
                    "Int" => Ok(Type::Int),
                    "Float" => Ok(Type::Float),
                    "String" => Ok(Type::String),
                    "Bool" => Ok(Type::Bool),
                    _ => Ok(Type::Custom(type_name.clone())),
                }
            }
            Some(token) => {
                Err(ParseError::UnexpectedToken {
                    expected: "type",
                    found: token.clone(),
                })
            }
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();

        while let Some(token) = self.peek() {
            if token == &Token::RParen {
                break;
            }

            if !params.is_empty() {
                self.expect(Token::Comma)?;
            }

            let name = match self.advance() {
                Some(Token::Identifier(name)) => name.clone(),
                Some(token) => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "parameter name",
                        found: token.clone(),
                    })
                }
                None => return Err(ParseError::UnexpectedEOF),
            };

            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Parameter {
                name,
                param_type,
                ownership: OwnershipType::Owned,
            });
        }

        Ok(params)
    }
}
