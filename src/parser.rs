use crate::ast::{Expr, Function, FunctionMap, FunctionVariant, Program};
use crate::bitstring::Bit;
use crate::pattern::{
    ConstLenPattern, ConstLenPatternElement, MultiPattern, Pattern, VarLenPattern,
};
use anyhow::{Context, Result as AnyResult};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "bitmachine.pest"]
struct BitMachineParser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

macro_rules! assert_rule {
    ($var:ident :: $rule:ident) => {
        assert_eq!($var.as_rule(), Rule::$rule, "Rule mismatch");
    };
    (:: $var:ident) => {
        assert_rule!($var::$var);
    };
}

pub fn parse(code: &str) -> AnyResult<Program> {
    let toplevel = BitMachineParser::parse(Rule::toplevel, code)
        .map_err(anyhow::Error::from)
        .with_context(|| String::from("Parse error"))?
        .next()
        .unwrap();
    assert_rule!(::toplevel);

    let program = toplevel.into_inner().next().unwrap();
    assert_rule!(::program);

    let mut map = FunctionMap::new();
    for (func_name, func_var) in program.into_inner().filter_map(parse_line) {
        map.entry(func_name.clone())
            .or_insert(Function {
                name: func_name,
                variants: vec![],
            })
            .variants
            .push(func_var);
    }

    Ok(Program { function_map: map })
}

fn parse_line(line: Pair<'_>) -> Option<(String, FunctionVariant)> {
    assert_rule!(::line);
    let inner = line.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::empty_line => None,
        Rule::func_def => Some(parse_func_def(inner)),
        _ => unreachable!(),
    }
}

fn parse_func_def(def: Pair<'_>) -> (String, FunctionVariant) {
    assert_rule!(def::func_def);
    let mut iter = def.into_inner();

    let name = String::from(parse_var_name(iter.next().unwrap()));
    let patterns = parse_patterns(iter.next().unwrap());
    let body = parse_expr(iter.next().unwrap());
    let var = FunctionVariant { patterns, body };
    (name, var)
}

fn parse_var_name(var_name: Pair<'_>) -> &str {
    assert_rule!(::var_name);
    var_name.as_str()
}

fn parse_patterns(patterns: Pair<'_>) -> MultiPattern {
    assert_rule!(::patterns);
    MultiPattern(patterns.into_inner().map(parse_pattern).collect())
}

fn parse_expr(expr: Pair<'_>) -> Expr {
    assert_rule!(::expr);
    let inner = expr.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr_call => parse_expr_call(inner),
        Rule::expr_single => parse_expr_single(inner),
        _ => unreachable!(),
    }
}

fn parse_expr_call(call: Pair<'_>) -> Expr {
    assert_rule!(call::expr_call);
    let mut iter = call.into_inner().map(parse_expr_single);
    let callee = iter.next().unwrap();
    let args = iter.collect();
    Expr::Call {
        callee: Box::new(callee),
        args,
    }
}

fn parse_expr_single(expr: Pair<'_>) -> Expr {
    assert_rule!(expr::expr_single);
    let inner = expr.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr_cat => parse_expr_cat(inner),
        Rule::expr_atomic => parse_expr_atomic(inner),
        _ => unreachable!(),
    }
}

fn parse_expr_cat(expr: Pair<'_>) -> Expr {
    assert_rule!(expr::expr_cat);
    Expr::Cat {
        children: expr.into_inner().map(parse_expr_atomic).collect(),
    }
}

fn parse_expr_atomic(expr: Pair<'_>) -> Expr {
    assert_rule!(expr::expr_atomic);
    let inner = expr.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr_paren => parse_expr_paren(inner),
        Rule::expr_literal => parse_expr_literal(inner),
        Rule::expr_name => parse_expr_name(inner),
        _ => unreachable!(),
    }
}

fn parse_expr_paren(expr: Pair<'_>) -> Expr {
    assert_rule!(expr::expr_paren);
    parse_expr(expr.into_inner().next().unwrap())
}

fn parse_expr_literal(expr: Pair<'_>) -> Expr {
    assert_rule!(expr::expr_literal);
    Expr::Literal(expr.as_str().parse().unwrap())
}

fn parse_expr_name(expr: Pair<'_>) -> Expr {
    assert_rule!(expr::expr_name);
    let inner = expr.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::var_name => Expr::Variable {
            name: String::from(parse_var_name(inner)),
            trampoline: true,
        },
        Rule::var_name_no_trampoline => Expr::Variable {
            name: String::from(parse_var_name(inner.into_inner().next().unwrap())),
            trampoline: false,
        },
        _ => unreachable!(),
    }
}

fn parse_pattern(pattern: Pair<'_>) -> Pattern {
    assert_rule!(::pattern);
    let inner = pattern.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::var_len_pattern => parse_var_len_pattern(inner),
        Rule::const_len_pattern => parse_const_len_pattern(inner).into(),
        Rule::empty_pattern => Pattern::empty(),
        _ => unreachable!(),
    }
}

fn parse_var_len_pattern(pattern: Pair<'_>) -> Pattern {
    assert_rule!(pattern::var_len_pattern);

    let mut maybe_left = None;
    let mut maybe_var_name = None;
    let mut maybe_right = None;

    for pair in pattern.into_inner() {
        match (maybe_var_name.is_some(), pair.as_rule()) {
            (false, Rule::const_len_pattern) => {
                maybe_left = Some(parse_const_len_pattern(pair));
            }
            (true, Rule::const_len_pattern) => {
                maybe_right = Some(parse_const_len_pattern(pair));
            }
            (false, Rule::var_name) => {
                maybe_var_name = Some(parse_var_name(pair));
            }
            _ => unreachable!(),
        }
    }

    let name = String::from(maybe_var_name.unwrap());

    match (maybe_left, maybe_right) {
        (None, None) => Pattern::Anything { name },
        (maybe_left, maybe_right) => VarLenPattern {
            left: maybe_left.unwrap_or(ConstLenPattern::empty()),
            right: maybe_right.unwrap_or(ConstLenPattern::empty()),
            bit_string_var_name: name,
        }
        .into(),
    }
}

fn parse_const_len_pattern(pattern: Pair<'_>) -> ConstLenPattern {
    assert_rule!(pattern::const_len_pattern);
    ConstLenPattern {
        elements: pattern
            .into_inner()
            .map(parse_const_len_pattern_item)
            .collect(),
    }
}

fn parse_const_len_pattern_item(item: Pair<'_>) -> ConstLenPatternElement {
    assert_rule!(item::const_len_pattern_item);
    let inner = item.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::pattern_const => ConstLenPatternElement::ConstBit(parse_pattern_const(inner)),
        Rule::pattern_bit => ConstLenPatternElement::AnyBit {
            var_name: String::from(parse_pattern_bit(inner)),
        },
        _ => unreachable!(),
    }
}

fn parse_pattern_const(pattern: Pair<'_>) -> Bit {
    assert_rule!(pattern::pattern_const);
    pattern.as_str().parse().unwrap()
}

fn parse_pattern_bit(pattern: Pair<'_>) -> &str {
    assert_rule!(pattern::pattern_bit);
    parse_var_name(pattern.into_inner().next().unwrap())
}
