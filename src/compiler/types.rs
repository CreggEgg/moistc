use std::{collections::HashMap, fmt::write};

use crate::parser::{Expr, Func, Op, Value};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Type {
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone)]
pub struct TypedFunc {
    body: TypedExpr,
    func_type: FuncType,
}

#[derive(Debug, Clone)]
pub struct FuncType {
    args: Vec<Type>,
    ret: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Value(Type, Value),
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
                    variables.insert(arg.name.clone(), self.string_to_type(&arg.arg_type));
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
            .map(|arg| self.string_to_type(&arg.arg_type))
            .collect::<Vec<Type>>();
        let body = self.expression_type(func.body, &mut variables);
        TypedFunc {
            body: body.clone(),
            func_type: FuncType {
                args,
                ret: get_type(body),
            },
        }
    }

    fn expression_type(&mut self, body: Expr, variables: &mut HashMap<String, Type>) -> TypedExpr {
        match body {
            Expr::Value(value) => TypedExpr::Value(self.value_type(value.clone()), value),
            Expr::Ident(ident) => TypedExpr::Ident(
                *variables
                    .get(&ident)
                    .expect(&format!("Undefined variable, {ident}")),
                ident,
            ),
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
                let ret = function.ret;
                let fn_args = function.args.clone();
                if !args.iter().enumerate().all(|(i, arg)| {
                    *fn_args.get(i).expect("Mismatched number of arguments")
                        == get_type(self.expression_type(*arg.clone(), variables))
                }) {
                    println!(
                        "passed in: {:?}, expected: {:?}",
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

    fn value_type(&mut self, value: crate::parser::Value) -> Type {
        match value {
            crate::parser::Value::Number(_) => Type::Int,
            Value::Bool(_) => Type::Bool,
        }
    }

    fn string_to_type(&mut self, type_name: &str) -> Type {
        match type_name {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            x => panic!("Unknown type, {}", x),
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
    }
}
