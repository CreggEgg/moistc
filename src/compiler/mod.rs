use cranelift::{
    codegen::{dbg, gimli::leb128::write},
    prelude::*,
};
use std::{borrow::Borrow, collections::HashMap, fs::File, sync::Arc};

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
use cranelift_module::{FuncId, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;

use crate::parser::{self, Expr, Func};

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

        Self {
            module: obj_module,
            isa,
            call_conv,
            function_builder_ctx: FunctionBuilderContext::new(),
            functions: HashMap::new(),
        }
    }

    pub fn compile_program(&mut self, funcs: Vec<Func>) -> Context {
        let mut ctx = self.module.make_context(); //for_function(self.main_function.clone()); //ew ugly clone please remove
        for (i, func) in funcs.iter().enumerate() {
            let mut signature = Signature::new(self.call_conv);
            signature.returns.push(AbiParam::new(I64));
            let fid = self
                .module
                .declare_function(&func.name, cranelift_module::Linkage::Export, &signature)
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
            function_builder.append_block_params_for_function_params(entry);

            let function_compiler = FunctionCompiler::new(
                function_builder,
                func.clone(),
                self.module,
                self.functions.clone(),
            );
            function_compiler.compile();

            ctx.func = function;

            self.module.define_function(fid, &mut ctx).unwrap();
        }
        ctx
    }

    // fn compile_function(&mut self, func: &Func, mut function: Function) {
    //     let ret = self.compile_expr(&func.body, function_builder);
    // }

    // fn finalize(&mut self) -> Context {
    //     ctx
    // }

    pub fn exec(&mut self, funcs: Vec<Func>) {
        let mut ctx = self.compile_program(funcs);

        dbg!(&ctx.func.name);

        let code = ctx
            .compile(self.isa.borrow(), &mut ControlPlane::default())
            .unwrap();

        let code_buffer = code.code_buffer();
        let mut buffer = memmap2::MmapOptions::new()
            .len(code_buffer.len())
            .map_anon()
            .unwrap();

        buffer.copy_from_slice(code_buffer);

        let buffer = buffer.make_exec().unwrap();

        let out = unsafe {
            let code_fn: unsafe extern "sysv64" fn() -> usize =
                std::mem::transmute(buffer.as_ptr());

            code_fn()
        };
        // let outfile = File::create("./main.o").unwrap();
        // ob
        println!("💦: {}", out);
    }

    pub fn build(mut self, funcs: Vec<Func>) {
        let _ = self.compile_program(funcs);
        let res = self.module.finish();
        let outfile = File::create("./main.o").unwrap();
        res.object.write_stream(outfile).unwrap();
    }
}

struct FunctionCompiler<'a> {
    builder: FunctionBuilder<'a>,
    func: Func,
    variables: HashMap<String, Variable>,
    functions: HashMap<String, FuncId>,
    module: ObjectModule,
}

impl<'a> FunctionCompiler<'a> {
    pub fn new(
        builder: FunctionBuilder<'a>,
        func: Func,
        module: ObjectModule,
        functions: HashMap<String, FuncId>,
    ) -> Self {
        Self {
            builder,
            func,
            variables: HashMap::new(),
            functions,
            module,
        }
    }

    pub fn compile(mut self) {
        let ret = self.compile_expr(self.func.body.clone());
        // dbg!(ret);
        self.builder.ins().return_(&[ret]);

        self.builder.finalize();
    }

    fn compile_expr(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Value(parser::Value::Number(x)) => self.builder.ins().iconst(I64, i64::from(x)),
            Expr::Ident(ident) => {
                let variable = self
                    .variables
                    .get(&ident)
                    .expect(&format!("Found undefined variable {}", ident));
                self.builder.use_var(*variable)
            }
            Expr::Operation(lhs, op, rhs) => {
                let lhs = self.compile_expr(*lhs);
                let rhs = self.compile_expr(*rhs);
                let ins = self.builder.ins();
                match op {
                    parser::Op::Add => ins.iadd(lhs, rhs),
                    parser::Op::Sub => ins.isub(lhs, rhs),
                    parser::Op::Mul => ins.imul(lhs, rhs),
                    parser::Op::Div => ins.sdiv(lhs, rhs),
                }
            }
            Expr::Def { ident, value, body } => {
                let value = self.compile_expr(*value);
                let variable = Variable::from_u32(self.variables.keys().len() as u32);
                self.builder.declare_var(variable, I64);
                self.builder.def_var(variable, value);
                self.variables.insert(ident, variable);
                self.compile_expr(*body)
            }
            Expr::FunctionCall(name, args) => {
                let func = self.module.declare_func_in_func(
                    *self
                        .functions
                        .get(&name)
                        .expect(&format!("Undefined function {}", name)),
                    self.builder.func,
                );
                let args = args.iter().map(|arg| self.compile_expr(**arg));
                let ret = self.builder.ins().call(func, args);
            }
        }
    }
}

// mod prelude {
//     pub extern "C" fn printchar(ch: i64) {
//         println!("💦: {}", char::from_u32(ch as u32).unwrap());
//     }
// }
