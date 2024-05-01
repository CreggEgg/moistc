use cranelift::{codegen::gimli::leb128::write, prelude::*};
use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use cranelift::{
    codegen::{
        control::ControlPlane,
        ir::{types::I32, AbiParam, Function, Signature, UserFuncName},
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
    main_function: Function,
    main_fid: FuncId,
    isa: Arc<dyn TargetIsa>,
    function_builder_ctx: FunctionBuilderContext,
    call_conv: CallConv,
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

        let mut main_signature = Signature::new(call_conv);
        main_signature.returns.push(AbiParam::new(I32));
        let main_fid = obj_module
            .declare_function("main", cranelift_module::Linkage::Export, &main_signature)
            .unwrap();

        let main_func = Function::with_name_signature(UserFuncName::user(0, 0), main_signature);

        Self {
            module: obj_module,
            main_function: main_func,
            main_fid,
            isa,
            call_conv,
            function_builder_ctx: FunctionBuilderContext::new(),
        }
    }

    pub fn compile_program(&mut self, funcs: Vec<Func>) {
        for (i, func) in funcs.iter().enumerate() {
            let mut sig = Signature::new(self.call_conv);
            sig.returns.push(AbiParam::new(I32));

            let mut function =
                Function::with_name_signature(UserFuncName::user(0, i.try_into().unwrap()), sig);

            let function_builder =
                FunctionBuilder::new(&mut function, &mut self.function_builder_ctx);
            let function_compiler = FunctionCompiler::new(function_builder, func.clone());
            function_compiler.compile();
        }
    }

    // fn compile_function(&mut self, func: &Func, mut function: Function) {
    //     let ret = self.compile_expr(&func.body, function_builder);
    // }

    fn finalize(&mut self) -> Context {
        let mut ctx = Context::for_function(self.main_function.clone()); //ew ugly clone please remove
        self.module
            .define_function(self.main_fid, &mut ctx)
            .unwrap();
        ctx
    }

    pub fn exec(&mut self) {
        let mut ctx = self.finalize();
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
        dbg!(out);
        // let outfile = File::create("./main.o").unwrap();
        // ob
    }
}

struct FunctionCompiler<'a> {
    builder: FunctionBuilder<'a>,
    func: Func,
    variables: HashMap<String, Variable>,
}

impl<'a> FunctionCompiler<'a> {
    pub fn new(builder: FunctionBuilder<'a>, func: Func) -> Self {
        Self {
            builder,
            func,
            variables: HashMap::new(),
        }
    }

    pub fn compile(mut self) {
        let entry = self.builder.create_block();

        self.builder.switch_to_block(entry);

        self.builder.seal_block(entry);

        let ret = self.compile_expr(self.func.body.clone());
        self.builder.ins().return_(&[ret]);

        self.builder.finalize();
    }

    fn compile_expr(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Value(parser::Value::Number(x)) => self.builder.ins().iconst(I32, i64::from(x)),
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
        }
    }
}
