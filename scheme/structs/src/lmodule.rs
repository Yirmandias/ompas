use crate::contextcollection::Context;
use crate::documentation::{Doc, DocCollection};
use crate::function::{
    AsyncNativeFn, AsyncNativeMutFn, LAsyncFn, LAsyncMutFn, LFn, LMutFn, NativeFn, NativeMutFn,
};
use crate::lvalue::LValue;
use crate::purefonction::PureFonctionCollection;
use std::any::Any;
use std::fmt::Display;

#[derive(Debug, Default, Clone)]
pub struct InitScheme(Vec<String>);

impl<T: ToString> From<Vec<T>> for InitScheme {
    fn from(vec: Vec<T>) -> Self {
        InitScheme(vec.iter().map(|x| x.to_string()).collect())
    }
}

impl InitScheme {
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    pub fn add(&mut self, expression: String) {
        self.0.push(expression)
    }

    pub fn inner(&self) -> Vec<&str> {
        self.0.iter().map(|x| x.as_str()).collect()
    }

    /*pub fn begin_lisp(&self) -> String {
        let mut str = String::new(); //"(begin ".to_string();
        self.0.iter().for_each(|x| {
            str.push_str(x.as_str());
            str.push('\n');
        });
        str
    }*/
}

/// Struct to define a Module, Library that will be loaded inside the Scheme Environment.
pub struct LModule {
    pub(crate) ctx: Context,
    pub(crate) bindings: Vec<(String, LValue)>,
    pub(crate) prelude: InitScheme,
    pub(crate) label: String,
    pub(crate) documentation: DocCollection,
    pub(crate) pure_fonctions: PureFonctionCollection,
    pub(crate) submodules: Vec<LModule>,
}

impl LModule {
    pub fn new<T: Any + Send + Sync>(ctx: T, label: impl Display, doc: impl Into<Doc>) -> Self {
        let mut module = Self {
            ctx: Context::new(ctx),
            bindings: vec![],
            prelude: Default::default(),
            label: label.to_string(),
            documentation: Default::default(),
            pure_fonctions: Default::default(),
            submodules: vec![],
        };
        let mut doc: Doc = doc.into();
        doc.verbose = Some(match doc.verbose {
            Some(verbose) => format!("{}\nElement(s) of the module:\n", verbose),
            None => "Element(s) of the module\n".to_string(),
        });
        module.documentation.insert(label, doc);
        module
    }

    pub fn add_doc(&mut self, label: impl Display, doc: impl Into<Doc>) {
        self.documentation.insert(label.to_string(), doc.into());
        self.documentation
            .get_mut(&self.label)
            .unwrap()
            .verbose
            .as_mut()
            .unwrap()
            .push_str(format!("- {}\n", label).as_str());
    }

    /// Add a function to the module.
    pub fn add_fn(&mut self, label: impl Display, fun: NativeFn, doc: impl Into<Doc>, pure: bool) {
        self.bindings.push((
            label.to_string(),
            LValue::Fn(LFn::new(fun, label.to_string())),
        ));
        if pure {
            self.pure_fonctions.add(label.to_string())
        }
        self.add_doc(label, doc);
    }

    pub fn add_mut_fn(&mut self, label: impl Display, fun: NativeMutFn, doc: impl Into<Doc>) {
        self.bindings.push((
            label.to_string(),
            LValue::MutFn(LMutFn::new(fun, label.to_string())),
        ));
        self.add_doc(label, doc);
    }

    pub fn add_async_fn(
        &mut self,
        label: impl Display,
        fun: AsyncNativeFn,
        doc: impl Into<Doc>,
        pure: bool,
    ) {
        self.bindings.push((
            label.to_string(),
            LValue::AsyncFn(LAsyncFn::new(fun, label.to_string())),
        ));
        if pure {
            self.pure_fonctions.add(label.to_string())
        }
        self.add_doc(label, doc);
    }

    pub fn add_async_mut_fn(
        &mut self,
        label: impl Display,
        fun: AsyncNativeMutFn,
        doc: impl Into<Doc>,
    ) {
        self.bindings.push((
            label.to_string(),
            LValue::AsyncMutFn(LAsyncMutFn::new(fun, label.to_string())),
        ));
        self.add_doc(label, doc);
    }

    pub fn add_lambda(
        &mut self,
        label: impl Display,
        expression: impl Display,
        doc: impl Into<Doc>,
    ) {
        self.prelude
            .add(format!("(define {} {})", label, expression));
        self.add_doc(label, doc);
    }

    pub fn add_macro(
        &mut self,
        label: impl Display,
        expression: impl Display,
        doc: impl Into<Doc>,
    ) {
        self.prelude
            .add(format!("(defmacro {} {})", label, expression));
        self.add_doc(label, doc);
    }

    pub fn add_submodule(&mut self, module: impl Into<LModule>) {
        let module: LModule = module.into();
        self.documentation
            .get_mut(&self.label)
            .unwrap()
            .verbose
            .as_mut()
            .unwrap()
            .push_str(format!("- {}\n", module.label).as_str());
        self.submodules.push(module)
    }

    /// Add a LValue to the prelude.
    pub fn add_prelude(&mut self, label: &str, lv: LValue, doc: impl Into<Doc>) {
        self.bindings.push((label.into(), lv));
        self.add_doc(label, doc);
    }
}

impl From<()> for LModule {
    fn from(t: ()) -> Self {
        Self {
            ctx: Context::new(t),
            bindings: vec![],
            prelude: Default::default(),
            label: "".to_string(),
            documentation: Default::default(),
            pure_fonctions: Default::default(),
            submodules: vec![],
        }
    }
}
