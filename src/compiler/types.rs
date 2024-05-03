use crate::parser::{Expr, Func};

pub enum Type {
    Int,
    Float,
    Bool,
}

pub struct FunctionType {
    params: Vec<Type>,
    returns: Vec<Type>,
}

pub struct TypedExpr(Type, Expr);
pub struct TypedFunc(FunctionType, Func);

pub fn generate_types(funcs: Vec<Func>) -> Vec<TypedExpr> {
    funcs
        .iter()
        .map(|func| {
            TypedFunc(
                FunctionType {
                    params: vec![],
                    returns: vec![],
                },
                *func,
            )
        })
        .collect::<Vec<TypedFunc>>()
}
