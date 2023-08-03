use {
    super::ExprNode,
    crate::{
        ast::{Expr, SelectItem, ToSqlUnquoted},
        ast_builder::expr::alias_as::ExprWithAliasNode,
        parse_sql::parse_select_item,
        result::{Error, Result},
        translate::translate_select_item,
    },
};

#[derive(Clone, Debug)]
pub enum SelectItemNode<'a> {
    SelectItem(SelectItem),
    Expr(ExprNode<'a>),
    Text(String),
    ExprWithAliasNode(ExprWithAliasNode<'a>),
}

impl<'a> From<SelectItem> for SelectItemNode<'a> {
    fn from(select_item: SelectItem) -> Self {
        Self::SelectItem(select_item)
    }
}

impl<'a> From<ExprNode<'a>> for SelectItemNode<'a> {
    fn from(expr_node: ExprNode<'a>) -> Self {
        Self::Expr(expr_node)
    }
}

impl<'a> From<&str> for SelectItemNode<'a> {
    fn from(select_item: &str) -> Self {
        Self::Text(select_item.to_owned())
    }
}

impl<'a> From<ExprWithAliasNode<'a>> for SelectItemNode<'a> {
    fn from(expr_node: ExprWithAliasNode<'a>) -> Self {
        Self::ExprWithAliasNode(expr_node)
    }
}

impl<'a> TryFrom<SelectItemNode<'a>> for SelectItem {
    type Error = Error;

    fn try_from(select_item_node: SelectItemNode<'a>) -> Result<Self> {
        match select_item_node {
            SelectItemNode::SelectItem(select_item) => Ok(select_item),
            SelectItemNode::Text(select_item) => {
                parse_select_item(select_item).and_then(|item| translate_select_item(&item))
            }
            SelectItemNode::Expr(expr_node) => {
                let expr = Expr::try_from(expr_node)?;
                let label = expr.to_sql_unquoted();

                Ok(SelectItem::Expr { expr, label })
            }
            SelectItemNode::ExprWithAliasNode(expr_node) => {
                let expr = Expr::try_from(expr_node.expr)?;
                let label = expr_node.alias.to_owned();

                Ok(SelectItem::Expr { expr, label })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::SelectItem,
        ast_builder::{col, SelectItemNode},
        parse_sql::parse_select_item,
        translate::translate_select_item,
    };

    fn test(actual: SelectItemNode, expected: &str) {
        let parsed = &parse_select_item(expected).expect(expected);
        let expected = translate_select_item(parsed);
        assert_eq!(actual.try_into(), expected);
    }

    #[test]
    fn select_item() {
        let actual = SelectItem::Wildcard.into();
        let expected = "*";
        test(actual, expected);

        let actual = "Foo.*".into();
        let expected = "Foo.*";
        test(actual, expected);

        let actual = "id as hello".into();
        let expected = "id as hello";
        test(actual, expected);

        let actual = col("id").into();
        let expected = "id";
        test(actual, expected);

        let actual = col("id").add(10).alias_as("col").into();
        let expected = "id + 10 AS col";
        test(actual, expected);
    }
}
