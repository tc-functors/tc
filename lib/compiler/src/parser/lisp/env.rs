use super::{
    builtin::Builtin,
    expr::Expr,
    symbol::Symbol,
};
use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    sync::Arc,
};

#[derive(Debug, Default, Clone)]
pub struct Env {
    bindings: Arc<HashMap<Expr, Arc<Expr>>>,
}

impl Env {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn bind_symbol(&mut self, symbol: &str, value: Expr) {
        self.bind(Expr::Symbol(Symbol::new(symbol)), value);
    }

    #[inline]
    pub fn bind_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, Vec<Expr>) -> Expr) {
        self.bind_symbol(symbol, Expr::Builtin(Builtin::new(f, symbol)));
    }

    #[inline]
    pub fn bind_lazy_builtin(&mut self, symbol: &'static str, f: fn(&mut Env, Vec<Expr>) -> Expr) {
        self.bind_symbol(
            symbol,
            Expr::Builtin(Builtin::new(f, symbol).with_lazy_eval(true)),
        );
    }

    #[inline]
    pub fn merge(&mut self, other: &Env) {
        for (k, v) in other.bindings.iter() {
            self.bind(k.clone(), (**v).clone());
        }
    }

    #[inline]
    pub fn alias(&mut self, from: impl Into<Symbol>, to: impl Into<Symbol>) {
        let from = from.into();
        let to = to.into();
        if let Some(value) = self.get(&Expr::Symbol(from)) {
            self.bind(Expr::Symbol(to), value.clone());
        }
    }

    pub fn get_bindings(&self) -> HashMap<Expr, Expr> {
        self.bindings
            .iter()
            .map(|(k, v)| (k.clone(), (**v).clone()))
            .collect()
    }

    #[inline]
    pub fn bind(&mut self, symbol: Expr, value: Expr) {
        if self.get(&symbol) == Some(&value) {
            return;
        }
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.insert(symbol, Arc::new(value));
    }

    #[inline]
    pub fn get(&self, symbol: &Expr) -> Option<&Expr> {
        self.bindings.get(symbol).map(|v| v.as_ref())
    }
    pub fn unbind(&mut self, symbol: &Expr) {
        let bindings = Arc::make_mut(&mut self.bindings);
        bindings.remove(symbol);
        self.bindings = Arc::new(bindings.clone());
    }

    #[inline]
    pub fn eval_str(&mut self, input: impl ToString) -> Result<Expr, String> {
        let input = input.to_string();
        let expr = Expr::parse(&input)?;
        Ok(self.eval(expr))
    }

    pub fn eval(&mut self, mut expr: Expr) -> Expr {
        use Expr::*;
        let saved_bindings = self.bindings.clone();
        let mut is_in_new_env = false;
        loop {
            if let Some(value) = self.get(&expr) {
                return value.clone();
            }

            match &expr {
                List(l) => {
                    if l.is_empty() {
                        return expr;
                    }

                    let mut args = l.clone();
                    let func = args.remove(0);
                    let func = self.eval(func);

                    match func {
                        Function(env, params, body) => {
                            // saved_bindings = self.bindings.clone();
                            is_in_new_env = true;
                            if let Some(new_env) = env {
                                self.merge(&new_env);
                            }

                            if params.len() != args.len() {
                                return Expr::Err(Box::new(Expr::String(format!(
                                    "Expected {} arguments, got {}",
                                    params.len(),
                                    args.len()
                                ))));
                            }

                            let args = args
                                .into_iter()
                                .map(|arg| self.eval(arg))
                                .collect::<Vec<_>>();

                            for (param, arg) in params.into_iter().zip(args.into_iter()) {
                                self.bind(param, arg);
                            }

                            expr = *body;
                        }
                        Builtin(f) => {
                            expr = (f.f)(self, args);
                            if !f.lazy_eval {
                                break;
                            }
                        }
                        Tree(t) => {
                            // Get the element of the tree
                            let key = self.eval(args.get(0).unwrap().clone());

                            expr = t.get(&key).cloned().unwrap_or(Expr::None);
                            break;
                        }
                        Map(m) => {
                            // Get the element of the map
                            let key = self.eval(args.get(0).unwrap().clone());
                            expr = m.get(&key).cloned().unwrap_or(Expr::None);
                            break;
                        }
                        Symbol(s) => {
                            if let Some(value) = self.get(&expr) {
                                expr = value.clone();
                            } else {
                                expr = Expr::Err(Box::new(Expr::String(format!(
                                    "Symbol {} not found",
                                    s.name()
                                ))));
                            }
                        }

                        _result => {
                            break;
                        }
                    }
                }
                Many(d) => {
                    if d.is_empty() {
                        return Expr::None;
                    }

                    // Eval the first expression
                    for e in d.iter().take(d.len() - 1) {
                        self.eval(e.clone());
                    }
                    expr = d.last().unwrap().clone();
                }
                Map(m) => {
                    let mut new_map = HashMap::new();
                    for (k, v) in m.iter() {
                        new_map.insert(k.clone(), self.eval(v.clone()));
                    }
                    expr = Expr::Map(new_map);
                    break;
                }
                Tree(t) => {
                    let mut new_tree = BTreeMap::new();
                    for (k, v) in t.iter() {
                        new_tree.insert(k.clone(), self.eval(v.clone()));
                    }
                    expr = Expr::Tree(new_tree);
                    break;
                }
                Quote(e) => {
                    expr = *e.clone();
                    break;
                }
                Function(Option::None, args, body) => {
                    // Replace the environment with a new one
                    let mut new_env = self.clone();
                    for arg in args.iter() {
                        new_env.unbind(arg);
                    }
                    expr = Function(Some(Box::new(new_env)), args.clone(), body.clone());
                    break;
                }
                _ => return expr,
            }
        }
        if is_in_new_env {
            self.bindings = saved_bindings;
        }
        expr
    }
}
