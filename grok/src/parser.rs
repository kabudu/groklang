use crate::ast::{AstNode, Param, Type};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<AstNode, String> {
        match program(input) {
            Ok((_, ast)) => Ok(ast),
            Err(e) => Err(format!("Parse error: {:?}", e)),
        }
    }
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphabetic() || c == '_')(input)
}

fn program(input: &str) -> IResult<&str, AstNode> {
    map(separated_list0(char(';'), function_def), AstNode::Program)(input)
}

fn function_def(input: &str) -> IResult<&str, AstNode> {
    map(
        tuple((
            tag("fn"),
            identifier,
            delimited(char('('), separated_list0(char(','), param), char(')')),
            opt(type_annotation),
            delimited(char('{'), tag(""), char('}')), // Placeholder for body
        )),
        |(_, name, params, ret_type, _)| AstNode::Function {
            name: name.to_string(),
            params,
            body: Box::new(AstNode::Program(vec![])), // Placeholder
            return_type: ret_type,
        },
    )(input)
}

fn param(input: &str) -> IResult<&str, Param> {
    map(
        pair(identifier, opt(preceded(char(':'), type_annotation))),
        |(name, ty)| Param {
            name: name.to_string(),
            ty,
        },
    )(input)
}

fn type_annotation(input: &str) -> IResult<&str, Type> {
    alt((
        map(tag("i32"), |_| Type::Int32),
        map(tag("f64"), |_| Type::Float64),
        map(tag("bool"), |_| Type::Bool),
        map(tag("String"), |_| Type::String),
    ))(input)
}

// Add more parsers as needed
