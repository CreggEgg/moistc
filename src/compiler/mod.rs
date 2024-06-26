use cranelift::{
    codegen::{
        dbg,
        gimli::leb128::write,
        ir::{stackslot::StackSize, types::I8},
    },
    prelude::*,
};
use std::{any::Any, borrow::Borrow, collections::HashMap, fs::File, sync::Arc};

use cranelift::{
    codegen::{
        control::ControlPlane,
        ir::{types::I64, AbiParam, Function, Signature, UserFuncName},
        isa::{self, CallConv, TargetIsa},
        settings::{self, Configurable},
        Context,
    },
    frontend::{FunctionBuilder, FunctionBuilderContext},
};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;

use crate::parser::{self, Expr, Func};

use self::types::{TypedExpr, TypedFunc, TypedValue};

pub mod types;

macro_rules! core_fn {
    ($name: expr, $functions: ident, $module: ident, $call_conv: ident) => {
        let mut signature = Signature::new($call_conv);
        signature.params.push(AbiParam::new(I64));
        signature.returns.push(AbiParam::new(I64));

        let fid = $module
            .declare_function($name, Linkage::Import, &signature)
            .unwrap();
        $functions.insert($name.to_string(), fid);
        println!("{}", $name.to_string());
    };
}

pub struct Compiler {
    module: ObjectModule,
    isa: Arc<dyn TargetIsa>,
    function_builder_ctx: FunctionBuilderContext,
    call_conv: CallConv,
    functions: HashMap<String, FuncId>,
}

// enum CompilerState {
//     Init,
//     Compiled,
// }

impl Compiler {
    pub fn new() -> Self {
        let mut settings_builder = settings::builder();
        settings_builder.enable("is_pic").unwrap();
        let flags = settings::Flags::new(settings_builder);

        let isa_builder = isa::lookup(Triple::host()).expect("Unsupported architecture");
        let isa = isa_builder.finish(flags).unwrap();
        let call_conv = isa.default_call_conv();

        let obj_builder = ObjectBuilder::new(
            isa.clone(),
            "main",
            cranelift_module::default_libcall_names(),
        );
        let mut obj_module = ObjectModule::new(obj_builder.unwrap());

        let mut functions = HashMap::new();

        // Add print function

        core_fn!("printint", functions, obj_module, call_conv);
        core_fn!("printchar", functions, obj_module, call_conv);
        core_fn!("printcharln", functions, obj_module, call_conv);
        core_fn!("printintln", functions, obj_module, call_conv);
        core_fn!("readchar", functions, obj_module, call_conv);

        Self {
            module: obj_module,
            isa,
            call_conv,
            function_builder_ctx: FunctionBuilderContext::new(),
            functions,
        }
    }

    pub fn compile_program(&mut self, funcs: Vec<TypedFunc>) -> Context {
        let mut ctx = self.module.make_context(); //for_function(self.main_function.clone()); //ew ugly clone please remove
        for (i, func) in funcs.iter().enumerate() {
            let mut signature = Signature::new(self.call_conv);
            signature.returns.push(AbiParam::new(I64));

            let mut args = HashMap::new();
            for (idx, arg) in func.args.iter().enumerate() {
                signature.params.push(AbiParam::new(I64));

                let var = Variable::new(idx);
                dbg!(&var);
                args.insert(arg.name.clone(), var);
            }

            let linkage = if func.name == String::from("main") {
                Linkage::Export
            } else {
                Linkage::Local
            };
            let fid = self
                .module
                .declare_function(&func.name, linkage, &signature)
                .unwrap();

            // dbg!(&signature);

            let mut function = Function::with_name_signature(
                UserFuncName::user(0, i.try_into().unwrap()),
                signature,
            );

            let mut function_builder =
                FunctionBuilder::new(&mut function, &mut self.function_builder_ctx);

            let entry = function_builder.create_block();

            function_builder.switch_to_block(entry);

            function_builder.seal_block(entry);

            let function_compiler = FunctionCompiler::new(
                function_builder,
                func.clone(),
                &mut self.module,
                self.functions.clone(),
                args,
            );
            function_compiler.compile(entry);

            self.functions.insert(func.name.clone(), fid);

            println!("{}", function.clone());
            ctx.func = function;

            self.module.define_function(fid, &mut ctx).unwrap();
            ctx.clear();
        }
        ctx
    }

    // fn compile_function(&mut self, func: &Func, mut function: Function) {
    //     let ret = self.compile_expr(&func.body, function_builder);
    // }

    // fn finalize(&mut self) -> Context {
    //     ctx
    // }

    // pub fn exec(&mut self, funcs: Vec<Func>) {
    //     let mut ctx = self.compile_program(funcs);
    //
    //     dbg!(&ctx.func.name);
    //
    //     let code = ctx
    //         .compile(self.isa.borrow(), &mut ControlPlane::default())
    //         .unwrap();
    //
    //     let code_buffer = code.code_buffer();
    //     let mut buffer = memmap2::MmapOptions::new()
    //         .len(code_buffer.len())
    //         .map_anon()
    //         .unwrap();
    //
    //     buffer.copy_from_slice(code_buffer);
    //
    //     let buffer = buffer.make_exec().unwrap();
    //
    //     let out = unsafe {
    //         let code_fn: unsafe extern "sysv64" fn() -> usize =
    //             std::mem::transmute(buffer.as_ptr());
    //
    //         code_fn()
    //     };
    //     // let outfile = File::create("./main.o").unwrap();
    //     // ob
    //     println!("💦: 5 {:b}", 5);
    //     println!("💦: {} {:b}", out, out);
    // }

    pub fn build(mut self, funcs: Vec<TypedFunc>) {
        let _ = self.compile_program(funcs);
        let res = self.module.finish();
        let outfile = File::create("./main.o").unwrap();
        res.object.write_stream(outfile).unwrap();
    }
}

struct FunctionCompiler<'a> {
    builder: FunctionBuilder<'a>,
    func: TypedFunc,
    variables: HashMap<String, Variable>,
    functions: HashMap<String, FuncId>,
    module: &'a mut ObjectModule,
}

impl<'a> FunctionCompiler<'a> {
    pub fn new(
        builder: FunctionBuilder<'a>,
        func: TypedFunc,
        module: &'a mut ObjectModule,
        functions: HashMap<String, FuncId>,
        variables: HashMap<String, Variable>,
    ) -> Self {
        dbg!(&variables);
        Self {
            builder,
            func,
            variables,
            functions,
            module,
        }
    }

    pub fn compile(mut self, block: Block) {
        self.builder.append_block_params_for_function_params(block);
        for (i, (_, arg)) in self.variables.iter().enumerate() {
            self.builder.declare_var(*arg, I64);
            if let Some(param) = self.builder.block_params(block).get(i) {
                self.builder.def_var(*arg, *param);
            }
        }
        dbg!(self.builder.block_params(block));

        let returning = self.compile_expr(self.func.body.clone());
        // dbg!(ret);
        dbg!(returning);
        self.builder.ins().return_(&[returning]);
        self.builder.seal_all_blocks();

        self.builder.finalize();
    }

    fn compile_expr(&mut self, expr: TypedExpr) -> Value {
        match expr {
            TypedExpr::Len(arr) => {
                let arr = self.compile_expr(*arr);
                self.builder.ins().load(I64, MemFlags::new(), arr, 0)
            }
            TypedExpr::Value(r#type, TypedValue::Number(x)) => {
                self.builder.ins().iconst(I64, i64::from(x))
            }
            TypedExpr::Value(r#type, TypedValue::Bool(x)) => {
                self.builder.ins().iconst(I64, if x { 1 } else { 0 })
            }
            TypedExpr::Value(r#type, TypedValue::Array(x)) => {
                let slot = self.construct_array(x);
                self.builder.ins().stack_addr(I64, slot, 0)
            }
            TypedExpr::Index {
                target,
                index,
                contained_type,
            } => {
                let target = self.compile_expr(*target);
                let index = self.compile_expr(*index);
                let value_size = self.builder.ins().iconst(I64, 64);
                let offset = self.builder.ins().imul(value_size, index);
                let offset = self.builder.ins().iadd(offset, value_size);
                let stack_ptr = self.builder.ins().iadd(offset, target);

                self.builder.ins().load(I64, MemFlags::new(), stack_ptr, 0)
            }
            TypedExpr::Ident(r#type, ident) => {
                dbg!(&self.variables);
                let variable = self
                    .variables
                    .get(&ident)
                    .expect(&format!("Found undefined variable {}", ident));
                println!("{}: {:?}", &ident, &variable);
                dbg!(self.builder.use_var(*variable))
            }
            TypedExpr::Operation(r#type, lhs, op, rhs) => {
                let lhs = self.compile_expr(*lhs);
                let rhs = self.compile_expr(*rhs);
                let ins = self.builder.ins();
                match op {
                    parser::Op::Add => ins.iadd(lhs, rhs),
                    parser::Op::Sub => ins.isub(lhs, rhs),
                    parser::Op::Mul => ins.imul(lhs, rhs),
                    parser::Op::Div => ins.sdiv(lhs, rhs),
                    _ => self.compile_comparsion(op, lhs, rhs),
                }
            }
            TypedExpr::Def { ident, value } => {
                let value = self.compile_expr(*value);
                let variable = Variable::from_u32(self.variables.keys().len() as u32);
                self.builder.declare_var(variable, I64);
                self.builder.def_var(variable, value);
                self.variables.insert(ident, variable);
                value
            }
            TypedExpr::FunctionCall(r#type, name, args) => {
                dbg!(&self.functions);

                let func = self.module.declare_func_in_func(
                    *self
                        .functions
                        .get(&name)
                        .expect(&format!("Undefined function {}", name)),
                    self.builder.func,
                );

                let args = args
                    .iter()
                    .map(|arg| {
                        dbg!(&arg);
                        dbg!(self.compile_expr(*arg.clone()))
                    })
                    .collect::<Vec<Value>>();
                let ret = self.builder.ins().call(func, &args);
                let recieved = self.builder.inst_results(ret);
                dbg!(recieved);
                recieved[0]
            }
            TypedExpr::Then { lhs, rhs } => {
                let _ = self.compile_expr(*lhs);
                self.compile_expr(*rhs)
            }
            TypedExpr::Each {
                body,
                ident,
                target,
            } => {
                let max = self.compile_expr(*target);

                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();
                let init = self.builder.ins().iconst(I64, 1);

                self.builder.ins().jump(header_block, &[init]);

                self.builder.append_block_param(header_block, I64);
                self.builder.append_block_param(body_block, I64);

                self.builder.switch_to_block(header_block);
                let i = self.builder.block_params(header_block)[0];
                let cond = self.builder.ins().icmp(IntCC::SignedGreaterThan, i, max);
                self.builder
                    .ins()
                    .brif(cond, exit_block, &[], body_block, &[i]);
                self.builder.switch_to_block(body_block);
                let var = Variable::new(self.variables.keys().len());
                self.builder.declare_var(var, I64);
                self.builder
                    .def_var(var, self.builder.block_params(body_block)[0]);
                self.variables.insert(ident, var);
                self.compile_expr(*body);
                let i = self.builder.use_var(var);
                let i = self.builder.ins().iadd(i, init);
                self.builder.ins().jump(header_block, &[i]);

                self.builder.switch_to_block(exit_block);

                self.builder.seal_block(header_block);
                self.builder.seal_block(body_block);
                self.builder.seal_block(exit_block);

                self.builder.ins().iconst(I64, 0)
            }
            TypedExpr::IfThen {
                condition,
                then,
                other,
            } => {
                let condition_value = self.compile_expr(*condition);

                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                self.builder.append_block_param(merge_block, I64);

                self.builder
                    .ins()
                    .brif(condition_value, then_block, &[], else_block, &[]);

                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                let then_return = self.compile_expr(*then);

                self.builder.ins().jump(merge_block, &[then_return]);

                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                let else_return = self.compile_expr(*other);

                self.builder.ins().jump(merge_block, &[else_return]);

                self.builder.switch_to_block(merge_block);

                self.builder.seal_block(merge_block);

                let phi = self.builder.block_params(merge_block)[0];

                phi
            }
        }
    }

    fn compile_comparsion(&mut self, op: parser::Op, lhs: Value, rhs: Value) -> Value {
        let comp = match op {
            parser::Op::Ge => self
                .builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
            parser::Op::Le => self
                .builder
                .ins()
                .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
            parser::Op::Gt => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
            parser::Op::Lt => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
            parser::Op::Eq => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
            parser::Op::Neq => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
            _ => self.builder.ins().iconst(I8, 0),
        };
        self.builder.ins().sextend(I64, comp)
    }

    fn construct_array(&mut self, x: Vec<TypedExpr>) -> codegen::ir::StackSlot {
        let slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            64 * (x.len() as u32),
        ));
        let len = self.builder.ins().iconst(I64, x.len() as i64);
        self.builder.ins().stack_store(len, slot, 0);
        for i in 0..x.len() {
            let value = self.compile_expr(x[i].clone());
            self.builder
                .ins()
                .stack_store(value, slot, ((i + 1) as i32) * 64);
        }
        slot
    }
}

fn drop_size_of(contained_type: types::Type) -> i64 {
    todo!()
}

// mod prelude {
//     pub extern "C" fn printchar(ch: i64) {
//         println!("💦: {}", char::from_u32(ch as u32).unwrap());
//     }
// }
