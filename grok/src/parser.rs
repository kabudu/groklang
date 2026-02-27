use crate::ast::{AstNode, MatchArm, Param, Pattern, Span, Type};
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_while, take_while1},
    character::complete::{char, digit1},
    combinator::{map, not, opt, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;
use std::fmt;

type Input<'a> = LocatedSpan<&'a str>;

fn span_from(input: Input) -> Span {
    Span {
        line: input.location_line() as usize,
        col: input.get_column() as usize,
    }
}

#[derive(Debug)]
pub struct Parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (line {}, col {}, offset {})",
            self.message, self.line, self.col, self.offset
        )
    }
}

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<AstNode, String> {
        self.parse_detailed(input).map_err(|e| e.to_string())
    }

    pub fn parse_detailed(&self, input: &str) -> Result<AstNode, ParseError> {
        let input = LocatedSpan::new(input);
        match nom::combinator::all_consuming(program)(input) {
            Ok((_, ast)) => Ok(ast),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(ParseError {
                message: "Parse error".to_string(),
                line: e.input.location_line() as usize,
                col: e.input.get_column(),
                offset: e.input.location_offset(),
            }),
            Err(nom::Err::Incomplete(_)) => Err(ParseError {
                message: "Parse error: incomplete input".to_string(),
                line: 0,
                col: 0,
                offset: 0,
            }),
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
    let (input, first) = take_while1(|c: char| c.is_alphabetic() || c == '_')(input)?;
    let (input, rest) = take_while(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    Ok((input, format!("{}{}", first.fragment(), rest.fragment())))
}

fn float_literal(input: Input) -> IResult<Input, f64> {
    map(
        tuple((
            digit1,
            char('.'),
            digit1,
            opt(alt((tag("f32"), tag("f64")))),
        )),
        |(int_part, _, frac_part, _): (Input, char, Input, Option<Input>)| {
            format!("{}.{}", int_part.fragment(), frac_part.fragment())
                .parse()
                .unwrap()
        },
    )(input)
}

fn int_literal(input: Input) -> IResult<Input, i64> {
    alt((
        map(
            pair(
                preceded(tag("0x"), take_while1(|c: char| c.is_ascii_hexdigit())),
                opt(alt((tag("i32"), tag("i64")))),
            ),
            |(n, _): (Input, Option<Input>)| i64::from_str_radix(n.fragment(), 16).unwrap(),
        ),
        map(
            pair(
                preceded(tag("0b"), take_while1(|c: char| c == '0' || c == '1')),
                opt(alt((tag("i32"), tag("i64")))),
            ),
            |(n, _): (Input, Option<Input>)| i64::from_str_radix(n.fragment(), 2).unwrap(),
        ),
        map(
            pair(
                preceded(tag("0o"), take_while1(|c: char| ('0'..='7').contains(&c))),
                opt(alt((tag("i32"), tag("i64")))),
            ),
            |(n, _): (Input, Option<Input>)| i64::from_str_radix(n.fragment(), 8).unwrap(),
        ),
        map(
            pair(digit1, opt(alt((tag("i32"), tag("i64"))))),
            |(n, _): (Input, Option<Input>)| n.parse().unwrap(),
        ),
    ))(input)
}

fn string_literal(input: Input) -> IResult<Input, String> {
    delimited(
        char('"'),
        escaped_transform(
            is_not("\\\"\n"),
            '\\',
            alt((
                value("\\", tag("\\")),
                value("\"", tag("\"")),
                value("\n", tag("n")),
                value("\r", tag("r")),
                value("\t", tag("t")),
                value("\0", tag("0")),
            )),
        ),
        char('"'),
    )(input)
}

fn char_literal(input: Input) -> IResult<Input, char> {
    delimited(
        char('\''),
        alt((
            nom::character::complete::none_of("\\'\n"),
            preceded(
                char('\\'),
                alt((
                    value('\\', char('\\')),
                    value('\'', char('\'')),
                    value('\n', char('n')),
                    value('\r', char('r')),
                    value('\t', char('t')),
                    value('\0', char('0')),
                )),
            ),
        )),
        char('\''),
    )(input)
}

fn raw_string_literal(input: Input) -> IResult<Input, String> {
    map(
        preceded(
            tag("r"),
            delimited(
                char('"'),
                take_while(|c: char| c != '"' && c != '\n'),
                char('"'),
            ),
        ),
        |s: Input| s.to_string(),
    )(input)
}

fn byte_string_literal(input: Input) -> IResult<Input, Vec<u8>> {
    map(
        preceded(
            char('b'),
            delimited(
                char('"'),
                escaped_transform(
                    is_not("\\\"\n"),
                    '\\',
                    alt((
                        value("\\", tag("\\")),
                        value("\"", tag("\"")),
                        value("\n", tag("n")),
                        value("\r", tag("r")),
                        value("\t", tag("t")),
                        value("\0", tag("0")),
                    )),
                ),
                char('"'),
            ),
        ),
        |s: String| s.into_bytes(),
    )(input)
}

fn program(input: Input) -> IResult<Input, AstNode> {
    map(many0(ws(declaration)), AstNode::Program)(input)
}

fn declaration(input: Input) -> IResult<Input, AstNode> {
    alt((
        use_decl,
        module_def,
        function_def,
        struct_def,
        enum_def,
        trait_def,
        impl_def,
        actor_def,
        macro_rules_def,
        map(statement, |s| s),
    ))(input)
}

fn path(input: Input) -> IResult<Input, Vec<String>> {
    separated_list1(ws(tag("::")), ws(identifier))(input)
}

fn use_import(input: Input) -> IResult<Input, (String, Option<String>)> {
    map(
        pair(identifier, opt(preceded(ws(tag("as")), ws(identifier)))),
        |(name, alias)| (name, alias),
    )(input)
}

fn use_decl(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("use"),
            ws(path),
            opt(alt((
                map(preceded(ws(tag("::")), ws(char('*'))), |_| {
                    (None, Vec::new(), true)
                }),
                map(
                    preceded(
                        ws(tag("::")),
                        delimited(
                            ws(char('{')),
                            terminated(
                                separated_list1(ws(char(',')), ws(use_import)),
                                opt(ws(char(','))),
                            ),
                            ws(char('}')),
                        ),
                    ),
                    |imports| (None, imports, false),
                ),
                map(preceded(ws(tag("as")), ws(identifier)), |alias| {
                    (Some(alias), Vec::new(), false)
                }),
            ))),
            ws(char(';')),
        )),
        move |(_, path, suffix, _)| {
            let (alias, imports, glob) = suffix.unwrap_or((None, Vec::new(), false));
            AstNode::UseDecl {
            path,
            alias,
            imports,
            glob,
            span: start_span.clone(),
        }
        },
    )(input)
}

fn module_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("mod"),
            ws(identifier),
            ws(alt((
                map(ws(char(';')), |_| None),
                map(
                    delimited(ws(char('{')), many0(ws(declaration)), ws(char('}'))),
                    Some,
                ),
            ))),
        )),
        move |(_, name, maybe_items)| match maybe_items {
            Some(items) => AstNode::ModuleDef {
                name,
                items,
                span: start_span.clone(),
            },
            None => AstNode::ModuleDecl {
                name,
                span: start_span.clone(),
            },
        },
    )(input)
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
            opt(ws(tag("pub"))),
            tag("fn"),
            ws(identifier),
            delimited(char('('), separated_list0(char(','), ws(param)), char(')')),
            opt(preceded(
                ws(tag("->")),
                alt((map(ws(tag("()")), |_| Type::Unit), ws(type_annotation))),
            )),
            opt(preceded(ws(tag("where")), ws(where_clause))),
            ws(block),
        )),
        move |(pub_kw, _, name, params, ret_type, where_bounds, body)| {
            let mut decorators = Vec::new();
            if pub_kw.is_some() {
                decorators.push("__pub".to_string());
            }
            if let Some(bounds) = where_bounds {
                for (type_var, traits) in bounds {
                    decorators.push(format!("__where:{}:{}", type_var, traits.join("+")));
                }
            }
            AstNode::FunctionDef {
            name,
            params,
            return_type: ret_type,
            body: Box::new(body),
            decorators,
            span: start_span.clone(),
        }
        },
    )(input)
}

fn where_clause(input: Input) -> IResult<Input, Vec<(String, Vec<String>)>> {
    separated_list1(
        ws(char(',')),
        map(
            tuple((
                identifier,
                ws(char(':')),
                separated_list1(ws(char('+')), ws(identifier)),
            )),
            |(type_var, _, traits)| (type_var, traits),
        ),
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
            opt(preceded(
                ws(char(':')),
                separated_list1(ws(char('+')), ws(identifier)),
            )),
            delimited(char('{'), many0(ws(function_def)), char('}')),
        )),
        move |(_, name, bounds, methods)| AstNode::TraitDef {
            name,
            methods,
            bounds: bounds.unwrap_or_default(),
            span: start_span.clone(),
        },
    )(input)
}

fn impl_generic_param(input: Input) -> IResult<Input, (String, Vec<String>)> {
    map(
        pair(
            identifier,
            opt(preceded(
                ws(char(':')),
                separated_list1(ws(char('+')), ws(identifier)),
            )),
        ),
        |(name, bounds)| (name, bounds.unwrap_or_default()),
    )(input)
}

fn impl_generics(input: Input) -> IResult<Input, Vec<(String, Vec<String>)>> {
    delimited(
        ws(char('<')),
        separated_list1(ws(char(',')), ws(impl_generic_param)),
        ws(char('>')),
    )(input)
}

fn impl_type_head(input: Input) -> IResult<Input, (String, Vec<String>)> {
    map(
        pair(
            identifier,
            opt(delimited(
                ws(char('<')),
                separated_list1(ws(char(',')), ws(identifier)),
                ws(char('>')),
            )),
        ),
        |(name, args)| (name, args.unwrap_or_default()),
    )(input)
}

fn impl_def(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            tag("impl"),
            opt(ws(impl_generics)),
            ws(impl_type_head),
            opt(preceded(ws(tag("for")), ws(impl_type_head))),
            delimited(char('{'), many0(ws(function_def)), char('}')),
        )),
        move |(_, maybe_generics, first, maybe_for_type, methods)| {
            let generic_bounds = maybe_generics.unwrap_or_default();
            if let Some((for_type, for_type_params)) = maybe_for_type {
                AstNode::ImplBlock {
                    trait_name: Some(first.0),
                    for_type,
                    for_type_params,
                    generic_bounds: generic_bounds.clone(),
                    methods,
                    span: start_span.clone(),
                }
            } else {
                AstNode::ImplBlock {
                    trait_name: None,
                    for_type: first.0,
                    for_type_params: first.1,
                    generic_bounds,
                    methods,
                    span: start_span.clone(),
                }
            }
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
    assignment_expr(input)
}

fn assignment_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (input, left) = send_expr(input)?;
    if let Ok((input, op)) = ws(alt((
        tag("+="),
        tag("-="),
        tag("*="),
        tag("/="),
        tag("%="),
        tag("&="),
        tag("|="),
        tag("^="),
        tag("<<="),
        tag(">>="),
        terminated(tag("="), not(tag(">"))),
    )))(input)
    {
        let (input, right) = assignment_expr(input)?;
        Ok((
            input,
            AstNode::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
                span: start_span,
            },
        ))
    } else {
        Ok((input, left))
    }
}

fn send_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (input, left) = logical_or_expr(input)?;
    if let Ok((input, _)) = ws(tag("!"))(input) {
        let (input, right) = expression(input)?;
        Ok((
            input,
            AstNode::Send {
                target: Box::new(left),
                message: Box::new(right),
                span: start_span.clone(),
            },
        ))
    } else {
        Ok((input, left))
    }
}

fn logical_or_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = logical_and_expr(input)?;
    while let Ok((i_op, op_span)) = ws(tag("||"))(input) {
        let (i_rhs, right) = logical_and_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn logical_and_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = bitwise_or_expr(input)?;
    while let Ok((i_op, op_span)) = ws(tag("&&"))(input) {
        let (i_rhs, right) = bitwise_or_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn bitwise_or_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = bitwise_xor_expr(input)?;
    while let Ok((i_op, op_span)) = ws(terminated(tag("|"), not(alt((tag("|"), tag("="))))))(input)
    {
        let (i_rhs, right) = bitwise_xor_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn bitwise_xor_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = bitwise_and_expr(input)?;
    while let Ok((i_op, op_span)) = ws(terminated(tag("^"), not(tag("="))))(input) {
        let (i_rhs, right) = bitwise_and_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn bitwise_and_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = equality_expr(input)?;
    while let Ok((i_op, op_span)) = ws(terminated(tag("&"), not(alt((tag("&"), tag("="))))))(input)
    {
        let (i_rhs, right) = equality_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn equality_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = relational_expr(input)?;
    while let Ok((i_op, op_span)) = ws(alt((tag("=="), tag("!="))))(input) {
        let (i_rhs, right) = relational_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn relational_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = shift_expr(input)?;
    while let Ok((i_op, op_span)) = ws(alt((tag("<="), tag(">="), tag("<"), tag(">"))))(input) {
        let (i_rhs, right) = shift_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn shift_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = additive_expr(input)?;
    while let Ok((i_op, op_span)) = ws(alt((
        terminated(tag("<<"), not(tag("="))),
        terminated(tag(">>"), not(tag("="))),
    )))(input)
    {
        let (i_rhs, right) = additive_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn additive_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = multiplicative_expr(input)?;
    while let Ok((i_op, op_span)) = ws(alt((
        terminated(tag("+"), not(tag("="))),
        terminated(tag("-"), not(tag("="))),
    )))(input)
    {
        let (i_rhs, right) = multiplicative_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
}

fn multiplicative_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    let (mut input, mut left) = unary_expr(input)?;
    while let Ok((i_op, op_span)) = ws(alt((
        terminated(tag("*"), not(tag("="))),
        terminated(tag("/"), not(tag("="))),
        terminated(tag("%"), not(tag("="))),
    )))(input)
    {
        let (i_rhs, right) = unary_expr(i_op)?;
        left = AstNode::BinaryOp {
            left: Box::new(left),
            op: op_span.to_string(),
            right: Box::new(right),
            span: start_span.clone(),
        };
        input = i_rhs;
    }
    Ok((input, left))
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
    let (mut input, first) = simple_pattern(input)?;
    let mut patterns = vec![first];
    while let Ok((i, _)) = ws(tag("|"))(input) {
        let (i2, pat) = simple_pattern(i)?;
        patterns.push(pat);
        input = i2;
    }

    if patterns.len() == 1 {
        Ok((input, patterns.pop().unwrap()))
    } else {
        Ok((input, Pattern::Or(patterns)))
    }
}

fn simple_pattern(input: Input) -> IResult<Input, Pattern> {
    alt((
        enum_pattern,
        struct_pattern,
        tuple_pattern,
        map(ws(tag("_")), |_| Pattern::Underscore),
        map(ws(tag("true")), |_| Pattern::BoolLiteral(true)),
        map(ws(tag("false")), |_| Pattern::BoolLiteral(false)),
        map(float_literal, Pattern::FloatLiteral),
        map(string_literal, Pattern::StringLiteral),
        map(int_literal, Pattern::IntLiteral),
        map(identifier, |id| Pattern::Identifier(id)),
    ))(input)
}

fn tuple_pattern(input: Input) -> IResult<Input, Pattern> {
    map(
        tuple((
            ws(char('(')),
            ws(pattern),
            ws(char(',')),
            separated_list0(ws(char(',')), ws(pattern)),
            opt(ws(char(','))),
            ws(char(')')),
        )),
        |(_, first, _, rest, _, _)| {
            let mut items = vec![first];
            items.extend(rest);
            Pattern::Tuple(items)
        },
    )(input)
}

fn struct_pattern(input: Input) -> IResult<Input, Pattern> {
    map(
        tuple((
            identifier,
            delimited(
                ws(char('{')),
                separated_list0(
                    ws(char(',')),
                    pair(identifier, opt(preceded(ws(char(':')), ws(pattern)))),
                ),
                ws(char('}')),
            ),
        )),
        |(name, fields)| {
            let fields = fields
                .into_iter()
                .map(|(field, pat)| {
                    let p = pat.unwrap_or_else(|| Pattern::Identifier(field.clone()));
                    (field, p)
                })
                .collect();
            Pattern::Struct(name, fields)
        },
    )(input)
}

fn enum_pattern(input: Input) -> IResult<Input, Pattern> {
    map(
        tuple((
            identifier,
            ws(tag("::")),
            identifier,
            opt(delimited(ws(char('(')), ws(pattern), ws(char(')')))),
        )),
        |(enum_name, _, variant, payload)| Pattern::Enum(enum_name, variant, payload.map(Box::new)),
    )(input)
}

fn unary_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    alt((
        map(
            tuple((
                ws(alt((
                    tag("&mut"),
                    tag("&"),
                    tag("*"),
                    tag("!"),
                    tag("-"),
                    tag("+"),
                    tag("~"),
                ))),
                unary_expr,
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
        } else if let Ok((i, index)) = delimited(ws(char('[')), expression, ws(char(']')))(input) {
            left = AstNode::IndexAccess {
                object: Box::new(left),
                index: Box::new(index),
                span: start_span,
            };
            input = i;
        } else if let Ok((i, _)) = ws(char('?'))(input) {
            left = AstNode::TryOp {
                expr: Box::new(left),
                span: start_span,
            };
            input = i;
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
    let s5 = span.clone();
    let s6 = span.clone();
    let s7 = span.clone();
    let s8 = span.clone();
    let s9 = span.clone();
    alt((
        macro_call,
        if_expr,
        match_expr,
        receive_expr,
        spawn_expr,
        closure_expr,
        tuple_literal,
        array_literal,
        struct_literal,
        map(ws(tag("true")), move |_| {
            AstNode::BoolLiteral(true, s1.clone())
        }),
        map(ws(tag("false")), move |_| {
            AstNode::BoolLiteral(false, s2.clone())
        }),
        map(float_literal, move |f| AstNode::FloatLiteral(f, s5.clone())),
        map(char_literal, move |c| AstNode::CharLiteral(c, s9.clone())),
        map(string_literal, move |s| {
            AstNode::StringLiteral(s, s6.clone())
        }),
        map(raw_string_literal, move |s| {
            AstNode::StringLiteral(s, s8.clone())
        }),
        map(byte_string_literal, move |b| {
            AstNode::ByteStringLiteral(b, s7.clone())
        }),
        map(identifier, move |id| AstNode::Identifier(id, s3.clone())),
        map(int_literal, move |n| AstNode::IntLiteral(n, s4.clone())),
        delimited(char('('), expression, char(')')),
    ))(input)
}

fn tuple_literal(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            ws(char('(')),
            ws(expression),
            ws(char(',')),
            separated_list0(ws(char(',')), ws(expression)),
            opt(ws(char(','))),
            ws(char(')')),
        )),
        move |(_, first, _, rest, _, _)| {
            let mut items = vec![first];
            items.extend(rest);
            AstNode::TupleLiteral(items, start_span.clone())
        },
    )(input)
}

fn closure_params(input: Input) -> IResult<Input, Vec<Param>> {
    delimited(
        ws(char('|')),
        separated_list0(ws(char(',')), ws(param)),
        ws(char('|')),
    )(input)
}

fn closure_expr(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        tuple((
            opt(terminated(tag("move"), skip_ws_and_comments)),
            ws(closure_params),
            opt(preceded(ws(tag("->")), ws(type_annotation))),
            ws(alt((block, expression))),
        )),
        move |(move_kw, params, return_type, body)| AstNode::Closure {
            params,
            return_type,
            body: Box::new(body),
            is_move: move_kw.is_some(),
            span: start_span.clone(),
        },
    )(input)
}

fn array_literal(input: Input) -> IResult<Input, AstNode> {
    let start_span = span_from(input);
    map(
        delimited(
            ws(char('[')),
            separated_list0(ws(char(',')), expression),
            ws(char(']')),
        ),
        move |items| AstNode::ArrayLiteral(items, start_span.clone()),
    )(input)
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
        map(tag("()"), |_| Type::Unit),
        map(
            tuple((
                ws(char('(')),
                ws(type_annotation),
                ws(char(',')),
                separated_list0(ws(char(',')), ws(type_annotation)),
                opt(ws(char(','))),
                ws(char(')')),
            )),
            |(_, first, _, rest, _, _)| {
                let mut items = vec![first];
                items.extend(rest);
                Type::Tuple(items)
            },
        ),
        map(tuple((tag("trait"), ws(identifier))), |(_, name)| {
            Type::Trait(name)
        }),
        map(
            tuple((tag("&"), opt(ws(tag("mut"))), ws(type_annotation))),
            |(_, mutable, inner)| Type::Reference(Box::new(inner), mutable.is_some()),
        ),
        map(
            tuple((
                identifier,
                delimited(
                    ws(char('<')),
                    separated_list0(ws(char(',')), ws(type_annotation)),
                    ws(char('>')),
                ),
            )),
            |(name, args)| Type::Generic(name, args),
        ),
        map(identifier, |id| match id.as_str() {
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
            | "u64" | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "str"
            | "String" => Type::Primitive(id),
            _ => Type::Variable(id),
        }),
    ))(input)
}
