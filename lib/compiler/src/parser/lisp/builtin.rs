use super::env::Env;
use super::expr::Expr;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub struct Builtin {
    pub f: fn(&mut Env, Vec<Expr>) -> Expr,
    pub name: &'static str,
    pub(crate) lazy_eval: bool,
}

impl Builtin {
    #[inline]
    pub fn new(f: fn(&mut Env, Vec<Expr>) -> Expr, name: &'static str) -> Self {
        Self {
            f,
            name,
            lazy_eval: false,
        }
    }

    #[inline]
    pub fn with_lazy_eval(self, lazy_eval: bool) -> Self {
        Self { lazy_eval, ..self }
    }

    #[inline]
    pub fn apply(&self, env: &mut Env, args: Vec<Expr>) -> Expr {
        (self.f)(env, args)
    }
}

impl Display for Builtin {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "<builtin {}>", self.name)
    }
}
