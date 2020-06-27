use boolinator::Boolinator;
use std::fmt::Debug;

use sqlparser::ast::{BinaryOperator, Expr, UnaryOperator};

use crate::executor::{select, BlendContext, FilterContext};
use crate::result::Result;
use crate::storage::Store;

use super::error::FilterError;

use crate::executor::evaluate;
use crate::executor::Parsed;

pub fn check_expr<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    filter_context: &'a FilterContext<'a>,
    expr: &'a Expr,
) -> Result<bool> {
    let parse = |expr| evaluate(storage, filter_context, expr);
    let check = |expr| check_expr(storage, filter_context, expr);

    match expr {
        Expr::BinaryOp { op, left, right } => {
            let zip_parse = || Ok((parse(left)?, parse(right)?));
            let zip_check = || Ok((check(left)?, check(right)?));

            match op {
                BinaryOperator::Eq => zip_parse().map(|(l, r)| l == r),
                BinaryOperator::NotEq => zip_parse().map(|(l, r)| l != r),
                BinaryOperator::And => zip_check().map(|(l, r)| l && r),
                BinaryOperator::Or => zip_check().map(|(l, r)| l || r),
                BinaryOperator::Lt => zip_parse().map(|(l, r)| l < r),
                BinaryOperator::LtEq => zip_parse().map(|(l, r)| l <= r),
                BinaryOperator::Gt => zip_parse().map(|(l, r)| l > r),
                BinaryOperator::GtEq => zip_parse().map(|(l, r)| l >= r),
                _ => Err(FilterError::Unimplemented.into()),
            }
        }
        Expr::UnaryOp { op, expr } => match op {
            UnaryOperator::Not => check(&expr).map(|v| !v),
            _ => Err(FilterError::Unimplemented.into()),
        },
        Expr::Nested(expr) => check(&expr),
        Expr::InList {
            expr,
            list,
            negated,
        } => {
            let negated = *negated;
            let target = parse(expr)?;

            list.iter()
                .filter_map(|expr| {
                    parse(expr).map_or_else(
                        |error| Some(Err(error)),
                        |parsed| (target == parsed).as_some(Ok(!negated)),
                    )
                })
                .next()
                .unwrap_or(Ok(negated))
        }
        Expr::InSubquery {
            expr,
            subquery,
            negated,
        } => {
            let negated = *negated;
            let target = parse(expr)?;

            select(storage, &subquery, Some(filter_context))?
                .map(|row| row?.take_first_value())
                .filter_map(|value| {
                    value.map_or_else(
                        |error| Some(Err(error)),
                        |value| (target == Parsed::ValueRef(&value)).as_some(Ok(!negated)),
                    )
                })
                .next()
                .unwrap_or(Ok(negated))
        }
        _ => Err(FilterError::Unimplemented.into()),
    }
}

pub fn check_blended_expr<T: 'static + Debug>(
    storage: &dyn Store<T>,
    filter_context: Option<&FilterContext<'_>>,
    blend_context: &BlendContext<'_, T>,
    expr: &Expr,
) -> Result<bool> {
    let BlendContext {
        table_alias,
        columns,
        row,
        next,
        ..
    } = blend_context;

    let filter_context = FilterContext::new(table_alias, &columns, &row, filter_context);

    match next {
        Some(blend_context) => {
            check_blended_expr(storage, Some(&filter_context), blend_context, expr)
        }
        None => check_expr(storage, &filter_context, expr),
    }
}
