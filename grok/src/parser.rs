use crate::ast::{AstNode, MatchArm, Param, Pattern, Span, Type};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map, opt, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;

type Input<'a> = LocatedSpan<&'a str>;

fn span_from(input: Input) -> Span {
    Span {
        line: input.location_line() as usize,
        col: input.get_column() as usize,
    }
}

#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<AstNode, String> {
        let input = LocatedSpan::new(input);
        match nom::combinator::all_consuming(program)(input) {
            Ok((_, ast)) => Ok(ast),
            Err(e) => Err(format!("Parse error: {:?}", e)),
        }
    }
}

fn ws<'a, F, O>(
    inner: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, O, nom::error::Error<Input<'a>>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O, nom::error::Error<Input<'a>>>,
{
    delimited(skip_ws_and_comments, inner, skip_ws_and_comments)
}

fn skip_ws_and_comments(input: Input) -> IResult<Input, ()> {
    let (input, _) = nom::character::complete::multispace0(input)?;
    let mut input = input;
    loop {
        if let Ok((i, _)) = tag::<_, _, nom::error::Error<Input>>("//")(input) {
            let (i, _) = take_while(|c: char| c != '\n')(i)?;
            let (i, _) = nom::character::complete::multispace0(i)?;
            input = i;
        } else {
            break;
        }
    }
    Ok((input, ()))
}

fn identifier(input: Input) -> IResult<Input, String> {
    map(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        |s: Input| s.to_string(),
    )(input)
}

fn program(input: Input) -> IResult<Input, AstNode> {
    map(many0(ws(declaration)), AstNode::Program)(input)
}

fn declaration(input: Input) -> IResult<Input, AstNode> {
    alt((
        function_def,
        struct_def,
        enum_def,
        trait_def,
        actor_def,
        macro_rules_def,
        map(statement, |s| s),
    ))(input)
}

fn macro_rules_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("macro_rules"),
            ws(tag("!")),
            ws(identifier),
            delimited(
                ws(char('{')),
                many0(ws(tuple((
                    delimited(ws(char('(')), pattern, ws(char(')'))),
                    ws(tag("=>")),
                    ws(delimited(
                        ws(char('{')),
                        many0(ws(statement)),
                        ws(char('}')),
                    )),
                )))),
                ws(char('}')),
            ),
        )),
        move |(_, _, name, rules)| AstNode::MacroDef {
            name,
            rules: rules
                .into_iter()
                .map(|(p, _, body)| (p, AstNode::Block(body)))
                .collect(),
            span: start_span.clone(),
        },
    )(input)
}

fn function_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("fn"),
            ws(identifier),
            delimited(char('('), separated_list0(char(','), ws(param)), char(')')),
            opt(preceded(ws(tag("->")), alt((
                map(ws(tag("()")), |_| Type::Primitive("()".to_string())),
                ws(type_annotation)
            )))),
            ws(block),
        )),
        move |(_, name, params, ret_type, body)| AstNode::FunctionDef {
            name,
            params,
            return_type: ret_type,
            body: Box::new(body),
            decorators: vec![],
            span: start_span.clone(),
        },
    )(input)
}

fn struct_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("struct"),
            ws(identifier),
            delimited(
                char('{'),
                separated_list0(
                    ws(char(',')),
                    ws(pair(identifier, preceded(ws(char(':')), type_annotation))),
                ),
                char('}'),
            ),
        )),
        move |(_, name, fields)| AstNode::StructDef {
            name,
            fields,
            generics: vec![],
            span: start_span.clone(),
        },
    )(input)
}

fn enum_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("enum"),
            ws(identifier),
            delimited(
                char('{'),
                separated_list0(
                    ws(char(',')),
                    ws(pair(
                        identifier,
                        opt(delimited(char('('), type_annotation, char(')'))),
                    )),
                ),
                char('}'),
            ),
        )),
        move |(_, name, variants)| AstNode::EnumDef {
            name,
            variants,
            generics: vec![],
            span: start_span.clone(),
        },
    )(input)
}

fn trait_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("trait"),
            ws(identifier),
            delimited(char('{'), many0(ws(function_def)), char('}')),
        )),
        move |(_, name, methods)| AstNode::TraitDef {
            name,
            methods,
            bounds: vec![],
            span: start_span.clone(),
        },
    )(input)
}

fn actor_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((tag("actor"), ws(identifier), ws(block))),
        move |(_, name, body)| AstNode::ActorDef {
            name,
            body: Box::new(body),
            span: start_span.clone(),
        },
    )(input)
}

fn statement(input: Input) -> IResult<Input, AstNode> {
    alt((
        let_stmt,
        return_stmt,
        break_stmt,
        continue_stmt,
        while_loop,
        for_loop,
        map(terminated(expression, opt(char(';'))), |e| e),
    ))(input)
}

fn let_stmt(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("let"),
            opt(ws(tag("mut"))),
            ws(identifier),
            opt(preceded(ws(char(':')), ws(type_annotation))),
            ws(char('=')),
            ws(expression),
            opt(ws(char(';'))),
        )),
        move |(_, mut_kw, name, ty, _, expr, _)| AstNode::LetStmt {
            name,
            mutable: mut_kw.is_some(),
            ty,
            expr: Box::new(expr),
            span: start_span.clone(),
        },
    )(input)
}

fn return_stmt(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        preceded(
            tag("return"),
            terminated(opt(ws(expression)), opt(char(';'))),
        ),
        move |val| AstNode::Return {
            value: val.map(Box::new),
            span: start_span.clone(),
        },
    )(input)
}

fn break_stmt(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    value(
        AstNode::Break { span: start_span },
        terminated(tag("break"), opt(char(';'))),
    )(input)
}

fn continue_stmt(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    value(
        AstNode::Continue { span: start_span },
        terminated(tag("continue"), opt(char(';'))),
    )(input)
}

fn block(input: Input) -> IResult<Input, AstNode> {
    map(
        delimited(char('{'), many0(ws(statement)), char('}')),
        AstNode::Block,
    )(input)
}

fn while_loop(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((tag("while"), ws(expression), ws(block))),
        move |(_, cond, body)| AstNode::WhileLoop {
            condition: Box::new(cond),
            body: Box::new(body),
            span: start_span.clone(),
        },
    )(input)
}

fn for_loop(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("for"),
            ws(identifier),
            ws(tag("in")),
            ws(expression),
            ws(block),
        )),
        move |(_, var, _, iterable, body)| AstNode::ForLoop {
            var,
            iterable: Box::new(iterable),
            body: Box::new(body),
            span: start_span.clone(),
        },
    )(input)
}

fn expression(input: Input) -> IResult<Input, AstNode> {
    alt((
        if_expr,
        match_expr,
        receive_expr,
        spawn_expr,
        binary_expr,
        unary_expr,
        postfix_expr,
    ))(input)
}

fn unary_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    alt((
        map(
            tuple((
                ws(alt((tag("&mut"), tag("&"), tag("*"), tag("!"), tag("-")))),
                expression,
            )),
            move |(op, expr)| AstNode::UnaryOp {
                op: op.to_string(),
                operand: Box::new(expr),
                span: start_span.clone(),
            },
        ),
        postfix_expr,
    ))(input)
}

fn if_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("if"),
            ws(expression),
            ws(block),
            opt(preceded(ws(tag("else")), alt((if_expr, block)))),
        )),
        move |(_, cond, then_body, else_body)| AstNode::IfExpr {
            condition: Box::new(cond),
            then_body: Box::new(then_body),
            else_body: else_body.map(Box::new),
            span: start_span.clone(),
        },
    )(input)
}

fn match_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("match"),
            ws(expression),
            delimited(char('{'), many0(ws(match_arm)), char('}')),
        )),
        move |(_, scrutinee, arms)| AstNode::MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            span: start_span.clone(),
        },
    )(input)
}

fn receive_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("receive"),
            delimited(ws(char('{')), many0(ws(match_arm)), ws(char('}'))),
        )),
        move |(_, arms)| AstNode::Receive {
            arms,
            span: start_span.clone(),
        },
    )(input)
}

fn spawn_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("spawn"),
            ws(identifier),
            delimited(
                ws(char('{')),
                separated_list0(
                    ws(char(',')),
                    pair(identifier, preceded(ws(char(':')), expression)),
                ),
                ws(char('}')),
            ),
        )),
        move |(_, actor, args)| AstNode::Spawn {
            actor,
            args,
            span: start_span.clone(),
        },
    )(input)
}

fn match_arm(input: Input) -> IResult<Input, MatchArm> {
    map(
        tuple((
            ws(pattern),
            opt(preceded(ws(tag("if")), ws(expression))),
            ws(tag("=>")),
            ws(statement),
            opt(char(',')),
        )),
        |(pattern, guard, _, body, _)| MatchArm {
            pattern,
            guard,
            body,
        },
    )(input)
}

fn pattern(input: Input) -> IResult<Input, Pattern> {
    alt((
        map(ws(tag("_")), |_| Pattern::Underscore),
        map(ws(tag("true")), |_| Pattern::BoolLiteral(true)),
        map(ws(tag("false")), |_| Pattern::BoolLiteral(false)),
        map(digit1, |n: Input| Pattern::IntLiteral(n.parse().unwrap())),
        map(identifier, |id| Pattern::Identifier(id)),
    ))(input)
}

fn binary_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            postfix_expr,
            ws(alt((
                tag("+"),
                tag("-"),
                tag("*"),
                tag("/"),
                tag("=="),
                tag("!="),
                tag("!"),
            ))),
            expression,
        )),
        move |(left, op_span, right)| {
            let op = op_span.to_string();
            if op == "!" {
                AstNode::Send {
                    target: Box::new(left),
                    message: Box::new(right),
                    span: start_span.clone(),
                }
            } else {
                AstNode::BinaryOp {
                    left: Box::new(left),
                    op: op.to_string(),
                    right: Box::new(right),
                    span: start_span.clone(),
                }
            }
        },
    )(input)
}

fn postfix_expr(input: Input) -> IResult<Input, AstNode> {
    let (mut input, mut left) = primary_expr(input)?;

    loop {
        let start_span = span_from(input);
        if let Ok((i, _)) = ws(char('.'))(input) {
            let (i2, member) = identifier(i)?;
            left = AstNode::MemberAccess {
                object: Box::new(left),
                member,
                span: start_span,
            };
            input = i2;
        } else if let Ok((i, args)) = delimited(
            ws(char('(')),
            separated_list0(ws(char(',')), expression),
            ws(char(')')),
        )(input)
        {
            left = AstNode::FunctionCall {
                func: Box::new(left),
                args,
                span: start_span,
            };
            input = i;
        } else {
            break;
        }
    }

    Ok((input, left))
}

fn primary_expr(input: Input) -> IResult<Input, AstNode> {
    let span = span_from(input);
    let s1 = span.clone();
    let s2 = span.clone();
    let s3 = span.clone();
    let s4 = span.clone();
    alt((
        macro_call,
        struct_literal,
        map(ws(tag("true")), move |_| {
            AstNode::BoolLiteral(true, s1.clone())
        }),
        map(ws(tag("false")), move |_| {
            AstNode::BoolLiteral(false, s2.clone())
        }),
        map(identifier, move |id| AstNode::Identifier(id, s3.clone())),
        map(digit1, move |n: Input| {
            AstNode::IntLiteral(n.parse().unwrap(), s4.clone())
        }),
        delimited(char('('), expression, char(')')),
    ))(input)
}

fn struct_literal(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            identifier,
            delimited(
                ws(char('{')),
                separated_list0(
                    ws(char(',')),
                    pair(identifier, preceded(ws(char(':')), expression)),
                ),
                ws(char('}')),
            ),
        )),
        move |(name, fields)| AstNode::StructLiteral {
            name,
            fields,
            span: start_span.clone(),
        },
    )(input)
}

fn macro_call(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            identifier,
            char('!'),
            delimited(
                char('('),
                separated_list0(ws(char(',')), expression),
                char(')'),
            ),
        )),
        move |(name, _, args)| AstNode::MacroCall {
            name,
            args,
            span: start_span.clone(),
        },
    )(input)
}

fn param(input: Input) -> IResult<Input, Param> {
    let span = span_from(input);
    map(
        pair(identifier, opt(preceded(ws(char(':')), type_annotation))),
        move |(name, ty)| Param {
            name,
            ty,
            span: span.clone(),
        },
    )(input)
}

fn type_annotation(input: Input) -> IResult<Input, Type> {
    alt((
        map(tag("i32"), |_| Type::Primitive("i32".to_string())),
        map(tag("i64"), |_| Type::Primitive("i64".to_string())),
        map(tag("f64"), |_| Type::Primitive("f64".to_string())),
        map(tag("bool"), |_| Type::Primitive("bool".to_string())),
        map(tag("String"), |_| Type::Primitive("String".to_string())),
        map(identifier, |id| Type::Variable(id)),
    ))(input)
}
