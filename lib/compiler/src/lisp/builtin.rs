/*
 * Builtin functions are supplied to the interpreter as functions that are
 * defined in Rust. These functions can be called from the lisp environment,
 * and can be used to extend the language with new functionality.
 *
 * All special forms, operators, and standard library functions are implemented
 * as built-in functions. This allows you to create your own standard library
 * of functions, and to override the default behavior of the interpreter.
 *
 * Builtin functions can be defined with the `Builtin::new` constructor, which
 * takes a function pointer and a name for the function. You can also set the
 * `lazy_eval` flag to true, to make the function's return value lazy-evaluated.
 */

use super::expr::Expr;
use super::env::Env;
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
