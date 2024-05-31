use core::panic;
use std::{collections::HashMap, fmt::write, mem};

use crate::parser::{Arg, Expr, Func, Op, Value};

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Int,
    Float,
    Bool,
    Array(Box<Type>),
}

// i hate this but i cant think of how to get rid of this enum
#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    Number(i32),
    Bool(bool),
    Array(Vec<TypedExpr>),
}

#[derive(Debug, Clone)]
pub struct TypedFunc {
    pub name: String,
    pub args: Vec<Arg>,
    pub func_type: FuncType,
    pub body: TypedExpr,
}

#[derive(Debug, Clone)]
pub struct FuncType {
    args: Vec<Type>,
    ret: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Value(Type, TypedValue),
    Ident(Type, String),
    Operation(Type, Box<TypedExpr>, Op, Box<TypedExpr>),
    Def {
        ident: String,
        value: Box<TypedExpr>,
    },
    Then {
        lhs: Box<TypedExpr>,
        rhs: Box<TypedExpr>,
    },
    FunctionCall(Type, String, Vec<Box<TypedExpr>>),
    IfThen {
        condition: Box<TypedExpr>,
        then: Box<TypedExpr>,
        other: Box<TypedExpr>,
    },
    Index {
        target: Box<TypedExpr>,
        index: Box<TypedExpr>,
        contained_type: Type,
    },
}

pub struct TypeGenerator {
    functions: HashMap<String, FuncType>,
}

impl TypeGenerator {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        functions.insert(
            "printchar".into(),
            FuncType {
                args: vec![Type::Int],
                ret: Type::Int,
            },
        );
        functions.insert(
            "print".into(),
            FuncType {
                args: vec![Type::Int],
                ret: Type::Int,
            },
        );
        functions.insert(
            "printcharln".into(),
            FuncType {
                args: vec![Type::Int],
                ret: Type::Int,
            },
        );
        functions.insert(
            "println".into(),
            FuncType {
                args: vec![Type::Int],
                ret: Type::Int,
            },
        );
        functions.insert(
            "readchar".into(),
            FuncType {
                args: vec![Type::Int],
                ret: Type::Int,
            },
        );
        Self { functions }
    }
    pub fn generate_types(&mut self, funcs: Vec<Func>) -> Vec<TypedFunc> {
        funcs
            .iter()
            .map(|func| {
                let mut variables = HashMap::new();
                for arg in &func.args {
                    variables.insert(arg.name.clone(), arg.arg_type.clone());
                }
                let func_type = self.generate_function_type(func.clone(), variables);
                self.functions
                    .insert(func.name.clone(), func_type.func_type.clone());
                func_type
            })
            .collect::<Vec<TypedFunc>>()
    }

    fn generate_function_type(
        &mut self,
        func: Func,
        mut variables: HashMap<String, Type>,
    ) -> TypedFunc {
        let args = func
            .args
            .iter()
            .map(|arg| arg.arg_type.clone())
            .collect::<Vec<Type>>();
        let mut func = func.clone();
        let body = self.expression_type(func.body, &mut variables);
        TypedFunc {
            body: body.clone(),
            name: mem::take(&mut func.name),
            args: mem::take(&mut func.args),
            func_type: FuncType {
                args,
                ret: get_type(body),
            },
        }
    }

    fn expression_type(&mut self, body: Expr, variables: &mut HashMap<String, Type>) -> TypedExpr {
        match body {
            Expr::Value(value) => TypedExpr::Value(
                value_type(self.type_value(value.clone(), variables)),
                self.type_value(value, variables),
            ),
            Expr::Ident(ident) => TypedExpr::Ident(
                variables
                    .get(&ident)
                    .expect(&format!("Undefined variable, {ident}"))
                    .clone(),
                ident,
            ),
            Expr::Index { target, index } => {
                let target_type = self.expression_type(*target, variables);
                let index_type = self.expression_type(*index, variables);
                match (get_type(target_type.clone()), get_type(index_type.clone())) {
                    (Type::Array(contained), Type::Int) => TypedExpr::Index {
                        target: Box::new(target_type),
                        index: Box::new(index_type),
                        contained_type: *contained,
                    },
                    (invalid_arr, invalid_int) => {
                        println!(
                            "Expected Array<_> got: {:?} and expected Int got: {:?}",
                            invalid_arr, invalid_int
                        );
                        panic!("Types in index expression are not valid!");
                    }
                }
            }
            Expr::Operation(lhs, op, rhs) => TypedExpr::Operation(
                self.force_identical(lhs.clone(), rhs.clone(), variables),
                Box::new(self.expression_type(*lhs, variables)),
                op,
                Box::new(self.expression_type(*rhs, variables)),
            ),
            Expr::Def { ident, value } => {
                let var_type = self.expression_type(*value, variables);
                variables.insert(ident.clone(), get_type(var_type.clone()));
                TypedExpr::Def {
                    ident,
                    value: Box::new(var_type),
                }
            }
            Expr::Then { lhs, rhs } => {
                // let _ = self.expression_type(*lhs, variables);
                // self.expression_type(*rhs, variables).expr_type
                TypedExpr::Then {
                    lhs: Box::new(self.expression_type(*lhs, variables)),
                    rhs: Box::new(self.expression_type(*rhs, variables)),
                }
            }
            Expr::FunctionCall(name, args) => {
                let function = self
                    .functions
                    .get(&name)
                    .expect(&format!("Undefined function, {}", name));
                let ret = function.ret.clone();
                let fn_args = function.args.clone();
                if !args.iter().enumerate().all(|(i, arg)| {
                    *fn_args.get(i).expect("Mismatched number of arguments")
                        == get_type(self.expression_type(*arg.clone(), variables))
                }) {
                    println!(
                        "function: {}, passed in: {:?}, expected: {:?}",
                        name,
                        args.iter()
                            .map(|arg| { get_type(self.expression_type(*arg.clone(), variables)) })
                            .collect::<Vec<Type>>(),
                        fn_args
                    );
                    panic!("Function arguments did not match");
                }
                TypedExpr::FunctionCall(
                    ret,
                    name,
                    args.iter()
                        .map(|arg| Box::new(self.expression_type(*arg.clone(), variables)))
                        .collect::<Vec<Box<TypedExpr>>>(),
                )
            }
            Expr::IfThen {
                condition,
                ref then,
                ref other,
            } => {
                let condition = self.expression_type(*condition, variables);
                if (get_type(condition.clone()) != Type::Bool) {
                    panic!("Condition is not of type Bool");
                }
                self.force_identical(then.clone(), other.clone(), variables);
                TypedExpr::IfThen {
                    condition: Box::new(condition),
                    then: Box::new(self.expression_type(*then.clone(), variables)),
                    other: Box::new(self.expression_type(*other.clone(), variables)),
                }
            }
        }
    }

    fn force_identical(
        &mut self,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        variables: &mut HashMap<String, Type>,
    ) -> Type {
        let lhs = get_type(self.expression_type(*lhs, variables));
        let rhs = get_type(self.expression_type(*rhs, variables));
        if lhs == rhs {
            lhs
        } else {
            println!("lhs: {:#?}, rhs: {:#?}", lhs, rhs);
            panic!("The types of lhs and rhs are not equal");
        }
    }

    fn string_to_type(&mut self, type_name: &str) -> Type {
        match type_name {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            x => {
                if (x.starts_with("Array<")) {
                    Type::Array(Box::new(self.string_to_type(&x[6..(x.len() - 1)])))
                } else {
                    panic!("Unknown type, {}", x)
                }
            }
        }
    }

    fn type_value(&mut self, value: Value, variables: &mut HashMap<String, Type>) -> TypedValue {
        match value {
            Value::Number(x) => TypedValue::Number(x),
            Value::Bool(x) => TypedValue::Bool(x),
            Value::Array(x) => TypedValue::Array(
                x.iter()
                    .map(|el| self.expression_type(el.clone(), variables))
                    .collect::<Vec<TypedExpr>>(),
            ),
        }
    }
}

fn get_type(expr: TypedExpr) -> Type {
    match expr {
        TypedExpr::Value(r#type, _) => r#type,
        TypedExpr::Ident(r#type, _) => r#type,
        TypedExpr::Operation(r#type, _, _, _) => r#type,
        TypedExpr::Def { ident, value } => get_type(*value),
        TypedExpr::Then { lhs, rhs } => get_type(*rhs),
        TypedExpr::FunctionCall(r#type, _, _) => r#type,
        TypedExpr::IfThen {
            condition,
            then,
            other,
        } => get_type(*then),
        TypedExpr::Index {
            target,
            index,
            contained_type,
        } => contained_type,
    }
}
fn value_type(value: TypedValue) -> Type {
    match value {
        TypedValue::Number(_) => Type::Int,
        TypedValue::Bool(_) => Type::Bool,
        TypedValue::Array(inner) => Type::Array(Box::new(get_type(
            inner.get(0).expect("Unable to infer array type").clone(),
        ))),
    }
}
