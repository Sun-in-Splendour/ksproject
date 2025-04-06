use super::CodeSpan;
use std::{
    collections::HashMap,
    rc::Weak,
    sync::{Arc, Mutex},
};

type Astr = Arc<str>;
type L<T> = Arc<Mutex<T>>;
type W<T> = Weak<Mutex<T>>;

pub struct Analyzer {
    pub undef_fn_calls: HashMap<Astr, Vec<UndefFnCall>>,
    pub names: HashMap<Astr, usize>,
}

pub struct VarInfo {
    pub name: Astr,
    pub def_span: CodeSpan,
    pub use_spans: Vec<CodeSpan>,
}

pub struct FnInfo {
    pub name: Astr,
    pub def_span: CodeSpan,
    pub use_spans: Vec<CodeSpan>,

    pub params: Vec<Astr>,
    pub args_span: CodeSpan,
    pub is_vararg: bool,

    pub scope: Option<L<Scope>>,
}

pub struct UndefFnCall {
    pub name: Astr,
    pub use_span: CodeSpan,
    pub params_num: usize,
    pub args_span: CodeSpan,
}

pub enum Named {
    Var(VarInfo),
    Fn(FnInfo),
}

pub struct Scope {
    pub parent: Option<W<Scope>>,

    pub locals: HashMap<Astr, Named>,
    pub cells: HashMap<Astr, L<Named>>,
    pub outers: HashMap<Astr, L<Named>>,
}
