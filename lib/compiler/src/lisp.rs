mod builtin;
mod env;
mod expr;
mod parser;
mod symbol;

use env::Env;
use expr::Expr;
use std::io::BufRead;
use symbol::Symbol;
use colored::Colorize;
use rustyline::{
    DefaultEditor,
    Result,
    error::ReadlineError,
};


fn make_env() -> Env {
    let mut env = Env::new();

    env.bind_builtin("env", |env, args| {
        // Get the env as a map
        if args.is_empty() {
            return Expr::Map(env.get_bindings());
        }
        let a = env.eval(args[0].clone());
        env.get(&a).cloned().unwrap_or(Expr::None)
    });

    env.bind_builtin("+", |env, exprs| {
        let mut sum = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            // sum += env.eval(e);
            match (sum, e) {
                (Expr::None, b) => sum = b,
                (Expr::Int(a), Expr::Int(b)) => sum = Expr::Int(a + b),
                (Expr::Float(a), Expr::Float(b)) => sum = Expr::Float(a + b),
                (Expr::Int(a), Expr::Float(b)) => sum = Expr::Float(a as f64 + b),
                (Expr::Float(a), Expr::Int(b)) => sum = Expr::Float(a + b as f64),
                (Expr::String(a), Expr::String(b)) => sum = Expr::String(format!("{}{}", a, b)),
                (Expr::List(a), Expr::List(b)) => {
                    let mut list = a.clone();
                    list.extend(b);
                    sum = Expr::List(list);
                }
                (Expr::List(a), b) => {
                    let mut list = a.clone();
                    list.push(b);
                    sum = Expr::List(list);
                }
                (a, b) => return Expr::error(format!("Invalid expr {} + {}", a, b)),
            }
        }
        sum
    });
    env.alias("+", "add");

    env.bind_builtin("-", |env, exprs| {
        let mut diff = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            match (diff, e) {
                (Expr::None, b) => diff = b,
                (Expr::Int(a), Expr::Int(b)) => diff = Expr::Int(a - b),
                (Expr::Float(a), Expr::Float(b)) => diff = Expr::Float(a - b),
                (Expr::Int(a), Expr::Float(b)) => diff = Expr::Float(a as f64 - b),
                (Expr::Float(a), Expr::Int(b)) => diff = Expr::Float(a - b as f64),
                (a, b) => return Expr::error(format!("Invalid expr {} - {}", a, b)),
            }
        }
        diff
    });
    env.alias("-", "sub");

    env.bind_builtin("*", |env, exprs| {
        let mut product = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            match (product, e) {
                (Expr::None, b) => product = b,
                (Expr::Int(a), Expr::Int(b)) => product = Expr::Int(a * b),
                (Expr::Float(a), Expr::Float(b)) => product = Expr::Float(a * b),
                (Expr::Int(a), Expr::Float(b)) => product = Expr::Float(a as f64 * b),
                (Expr::Float(a), Expr::Int(b)) => product = Expr::Float(a * b as f64),
                (Expr::List(a), Expr::Int(b)) => {
                    let mut list = a.clone();
                    for _ in 0..b {
                        list.extend(a.clone());
                    }
                    product = Expr::List(list);
                }
                (a, b) => return Expr::error(format!("Invalid expr {} * {}", a, b)),
            }
        }
        product
    });
    env.alias("*", "mul");

    env.bind_builtin("/", |env, exprs| {
        let mut quotient = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            match (quotient, e) {
                (Expr::None, b) => quotient = b,
                (Expr::Int(a), Expr::Int(b)) => quotient = Expr::Int(a / b),
                (Expr::Float(a), Expr::Float(b)) => quotient = Expr::Float(a / b),
                (Expr::Int(a), Expr::Float(b)) => quotient = Expr::Float(a as f64 / b),
                (Expr::Float(a), Expr::Int(b)) => quotient = Expr::Float(a / b as f64),
                (a, b) => return Expr::error(format!("Invalid expr {} / {}", a, b)),
            }
        }
        quotient
    });
    env.alias("/", "div");

    env.bind_builtin("%", |env, exprs| {
        let mut quotient = Expr::default();
        for e in exprs {
            let e = env.eval(e.clone());
            match (quotient, e) {
                (Expr::None, b) => quotient = b,
                (Expr::Int(a), Expr::Int(b)) => quotient = Expr::Int(a % b),
                (Expr::Float(a), Expr::Float(b)) => quotient = Expr::Float(a % b),
                (Expr::Int(a), Expr::Float(b)) => quotient = Expr::Float(a as f64 % b),
                (Expr::Float(a), Expr::Int(b)) => quotient = Expr::Float(a % b as f64),
                (a, b) => return Expr::error(format!("Invalid expr {} % {}", a, b)),
            }
        }
        quotient
    });
    env.alias("%", "rem");

    env.bind_builtin("=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());

        Expr::Bool(a == b)
    });
    env.alias("=", "==");

    env.bind_builtin("!=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());

        Expr::Bool(a != b)
    });

    env.bind_builtin("<=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());

        Expr::Bool(a <= b)
    });

    env.bind_builtin(">=", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());

        Expr::Bool(a >= b)
    });

    env.bind_builtin("<", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());

        Expr::Bool(a < b)
    });

    env.bind_builtin(">", |env, exprs| {
        let a = env.eval(exprs[0].clone());
        let b = env.eval(exprs[1].clone());

        Expr::Bool(a > b)
    });

    env.bind_lazy_builtin("if", |env, exprs| {
        let cond = env.eval(exprs[0].clone());
        let then = exprs[1].clone();
        if exprs.len() < 3 {
            if cond == Expr::Bool(true) {
                then
            } else {
                Expr::None
            }
        } else {
            let else_ = exprs[2].clone();
            if cond == Expr::Bool(true) {
                then
            } else {
                else_
            }
        }
    });

    env.bind_builtin("define", |env, exprs| {
        let name = exprs[0].clone();
        let value = env.eval(exprs[1].clone());
        env.bind(name, value);
        Expr::None
    });

    env.bind_builtin("undefine", |env, exprs| {
        let name = exprs[0].clone();
        env.unbind(&name);
        Expr::None
    });

    env.bind_builtin("defun", |env, args| {
        let name = args[0].clone();
        let params = args[1].clone();
        let body = args[2].clone();
        if let Expr::List(params) = params {
            let f = env.eval(Expr::Function(None, params, Box::new(body)));
            env.bind(name, f);
            Expr::None
        } else {
            return Expr::error(format!("Invalid params {:?}", params));
        }
    });

    env.bind_builtin("println", |env, exprs| {
        for e in exprs {
            let e = env.eval(e.clone());

            match e {
                Expr::String(s) => print!("{}", s),
                Expr::Symbol(s) => print!("{}", s.name()),
                _ => print!("{}", e),
            }
        }
        println!();
        Expr::None
    });

    // env.bind_builtin("do", |env, exprs| {
    //     let mut result = Expr::default();
    //     for e in exprs {
    //         result = env.eval(e.clone());
    //     }
    //     result
    // });

    env.bind_lazy_builtin("do", |_env, exprs| {
        use std::sync::Arc;
        Expr::Many(Arc::new(Vec::from(exprs)))
    });

    env.bind_builtin("sqrt", |env, expr| {
        let e = env.eval(expr[0].clone());
        match e {
            Expr::Int(i) => Expr::Float((i as f64).sqrt()),
            Expr::Float(f) => Expr::Float(f.sqrt()),
            e => Expr::error(format!("Invalid expr sqrt {}", e)),
        }
    });

    env.bind_builtin("^", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        match (a, b) {
            (Expr::Int(a), Expr::Int(b)) => Expr::Float((a as f64).powf(b as f64)),
            (Expr::Float(a), Expr::Float(b)) => Expr::Float(a.powf(b)),
            (Expr::Int(a), Expr::Float(b)) => Expr::Float((a as f64).powf(b)),
            (Expr::Float(a), Expr::Int(b)) => Expr::Float(a.powf(b as f64)),
            (a, b) => Expr::error(format!("Invalid expr {} ^ {}", a, b)),
        }
    });

    env.alias("^", "pow");

    let lambda = |env: &mut Env, expr: Vec<Expr>| {
        let params = expr[0].clone();
        let body = expr[1].clone();
        if let Expr::List(params) = params {
            Expr::Function(Some(Box::new(env.clone())), params, Box::new(body))
        } else {
            return Expr::error(format!("Invalid params {:?}", params));
        }
    };
    env.bind_builtin("lambda", lambda);
    env.bind_builtin("\\", lambda);

    env.bind_builtin("apply", |env, expr| {
        let f = env.eval(expr[0].clone());
        let args = env.eval(expr[1].clone());
        if let Expr::List(args) = args {
            match f {
                Expr::Function(Some(mut env), params, body) => {
                    let mut new_env = env.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        new_env.bind(param.clone(), env.eval(arg.clone()));
                    }
                    new_env.eval(*body.clone())
                }
                Expr::Function(None, params, body) => {
                    let mut new_env = env.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        new_env.bind(param.clone(), env.eval(arg.clone()));
                    }
                    new_env.eval(*body.clone())
                }
                Expr::Builtin(f) => f.apply(&mut env.clone(), args),
                f => Expr::error(format!("Invalid function {f} apply {}", Expr::from(args))),
            }
        } else {
            Expr::error(format!("Invalid function {f} apply {}", Expr::from(args)))
        }
    });

    env.bind_builtin("cons", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        // Create a new list with a as the head and b as the tail.
        if let Expr::List(b) = b {
            let mut list = vec![a];
            list.extend(b);
            Expr::List(list)
        } else if b == Expr::None {
            Expr::List(vec![a])
        } else {
            Expr::List(vec![a, b])
        }
    });
    let head = |env: &mut Env, expr: Vec<Expr>| {
        let a = env.eval(expr[0].clone());
        if let Expr::List(a) = a {
            a[0].clone()
        } else {
            Expr::error(format!("Invalid head {a}"))
        }
    };
    let tail = |env: &mut Env, expr: Vec<Expr>| {
        let a = env.eval(expr[0].clone());
        if let Expr::List(a) = a {
            Expr::List(a[1..].to_vec())
        } else {
            Expr::error(format!("Invalid tail {a}"))
        }
    };

    env.bind_builtin("car", head);
    env.bind_builtin("head", head);
    env.bind_builtin("cdr", tail);
    env.bind_builtin("tail", tail);

    let format = |env: &mut Env, expr: Vec<Expr>| {
        let format = env.eval(expr[0].clone());
        // Collect the args
        let args = expr[1..].to_vec();

        let mut format = match format {
            Expr::String(s) => s,
            e => return Expr::error(format!("Invalid format {e}")),
        };

        // Find all of the format specifiers.
        let mut specifiers = vec![];
        for (i, c) in format.chars().enumerate() {
            if c == '{' {
                let mut j = i + 1;
                while j < format.len() {
                    if format.chars().nth(j).unwrap() == '}' {
                        break;
                    }
                    j += 1;
                }
                specifiers.push(format[i + 1..j].to_owned());
            }
        }

        // Replace the named specifiers with variables in the scope.
        for name in &specifiers {
            if name.is_empty() {
                continue;
            }
            let name = Expr::Symbol(Symbol::new(name));

            let value = env.eval(name.clone());
            let specifier = format!("{{{name}}}");
            match value {
                Expr::String(s) => {
                    format = format.replacen(&specifier, &s, 1);
                }
                other => {
                    format = format.replacen(&specifier, &other.to_string(), 1);
                }
            }
        }

        // Replace the empty specifiers with the args.
        let mut i = 0;
        for name in &specifiers {
            if !name.is_empty() {
                continue;
            }
            if i >= args.len() {
                return Expr::error("Too few arguments");
            }
            let specifier = format!("{{}}");
            let value = env.eval(args[i].clone());
            match value {
                Expr::String(s) => {
                    format = format.replacen(&specifier, &s, 1);
                }
                other => {
                    format = format.replacen(&specifier, &other.to_string(), 1);
                }
            }
            // format = format.replacen("{}", &args[i].to_string(), 1);
            i += 1;
        }

        if i < args.len() {
            return Expr::error("Too many arguments");
        }

        Expr::String(format)
    };

    env.bind_builtin("format", format);

    env.bind_builtin("list", |env, expr| {
        let mut list = vec![];
        for e in expr {
            list.push(env.eval(e.clone()));
        }
        Expr::List(list)
    });

    env.bind_builtin("append", |env, expr| {
        let mut list = vec![];
        for e in expr {
            let e = env.eval(e.clone());
            if let Expr::List(l) = e {
                list.extend(l);
            } else {
                return Expr::error(format!("Invalid append {e}"));
            }
        }
        Expr::List(list)
    });

    env.bind_builtin("eval", |env, expr| {
        let e = env.eval(expr[0].clone());
        env.eval(e)
    });

    env.bind_builtin("exit", |env, expr| match env.eval(expr[0].clone()) {
        Expr::Int(i) => std::process::exit(i as i32),
        Expr::String(s) => {
            eprintln!("{s}");
            std::process::exit(1);
        }
        e => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    });

    env.bind_builtin("quote", |_env, expr| expr[0].clone());

    env.bind_builtin("or", |env, expr| {
        for e in expr {
            let e = env.eval(e.clone());
            if e == Expr::Bool(true) {
                return Expr::Bool(true);
            }
        }
        Expr::Bool(false)
    });

    env.bind_builtin("and", |env, expr| {
        for e in expr {
            let e = env.eval(e.clone());
            if e == Expr::Bool(false) {
                return Expr::Bool(false);
            }
        }
        Expr::Bool(true)
    });

    env.bind_builtin("not", |env, expr| {
        let e = env.eval(expr[0].clone());
        match e {
            Expr::Bool(b) => Expr::Bool(!b),
            e => return Expr::error(format!("Invalid not {e}")),
        }
    });

    env.bind_builtin("len", |env, expr| {
        let e = env.eval(expr[0].clone());
        match e {
            Expr::String(s) => Expr::Int(s.len() as i64),
            Expr::List(l) => Expr::Int(l.len() as i64),
            Expr::Map(m) => Expr::Int(m.len() as i64),
            Expr::Tree(t) => Expr::Int(t.len() as i64),
            e => Expr::error(format!("Invalid len {e}")),
        }
    });

    env.bind_builtin("let", |env, expr| {
        let mut new_env = env.clone();
        let bindings = expr[0].clone();
        let body = expr[1].clone();
        match bindings {
            Expr::List(bindings) => {
                for binding in bindings {
                    if let Expr::List(binding) = binding {
                        let name = binding[0].clone();
                        let value = env.eval(binding[1].clone());
                        new_env.bind(name, value);
                    } else {
                        return Expr::error(format!("Invalid binding {binding}"));
                    }
                }
            }
            Expr::Map(bindings) => {
                for (name, value) in bindings {
                    new_env.bind(name, env.eval(value));
                }
            }
            Expr::Tree(bindings) => {
                for (name, value) in bindings {
                    new_env.bind(name, env.eval(value));
                }
            }
            bindings => return Expr::error(format!("Invalid bindings {bindings}")),
        }
        new_env.eval(body)
    });

    env.bind_builtin("get", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());

        match (a, b) {
            (Expr::String(a), Expr::Int(b)) => {
                Expr::String(a.chars().nth(b as usize).unwrap_or('\0').to_string())
            }
            (Expr::List(a), Expr::Int(b)) => a.get(b as usize).cloned().unwrap_or(Expr::None),
            (Expr::Map(a), Expr::Symbol(b)) => {
                // a.get(&b).cloned().unwrap_or(Expr::None)
                a.get(&Expr::Symbol(b.clone())).cloned().unwrap_or_else(|| {
                    a.get(&Expr::String(b.name().to_owned()))
                        .cloned()
                        .unwrap_or(Expr::None)
                })
            }
            (Expr::Map(a), b) => a.get(&b).cloned().unwrap_or(Expr::None),
            (Expr::Tree(a), b) => a.get(&b).cloned().unwrap_or(Expr::None),
            (a, b) => return Expr::error(format!("Invalid expr get {} {}", a, b)),
        }
    });
    env.alias("get", "@");

    env.bind_builtin("set", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        let c = env.eval(expr[2].clone());

        match (a, b) {
            (Expr::String(mut a), Expr::Int(b)) => {
                if b == a.len() as i64 {
                    a.push_str(&c.to_string());
                } else {
                    a = a
                        .chars()
                        .enumerate()
                        .map(|(i, c)| if i == b as usize { c } else { '\0' })
                        .collect();
                }
                Expr::String(a)
            }
            (Expr::List(mut a), Expr::Int(b)) => {
                if b as usize >= a.len() {
                    a.resize(b as usize + 1, Expr::None);
                }
                a[b as usize] = c;
                Expr::List(a)
            }
            (Expr::Map(mut a), b) => {
                a.insert(b, c);
                Expr::Map(a)
            }
            (Expr::Tree(mut a), b) => {
                a.insert(b, c);
                Expr::Tree(a)
            }
            (a, b) => return Expr::error(format!("Invalid expr set {} {} {}", a, b, c)),
        }
    });

    env.bind_builtin("zip", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());

        match (a, b) {
            (Expr::List(a), Expr::List(b)) => {
                let mut list = vec![];
                for (a, b) in a.into_iter().zip(b.into_iter()) {
                    list.push(Expr::List(vec![a, b]));
                }
                Expr::List(list)
            }
            (a, b) => return Expr::error(format!("Invalid expr zip {} {}", a, b)),
        }
    });

    // Convert a list of pairs into a map.
    env.bind_builtin("to-map", |env, expr| {
        let a = env.eval(expr[0].clone());
        match a {
            Expr::List(a) => {
                let mut map = std::collections::HashMap::new();
                for e in a {
                    if let Expr::List(e) = e {
                        if e.len() == 2 {
                            map.insert(e[0].clone(), e[1].clone());
                        } else {
                            return Expr::error(format!("Invalid pair {}", Expr::from(e)));
                        }
                    } else {
                        return Expr::error(format!("Invalid pair {}", Expr::from(e)));
                    }
                }
                Expr::Map(map)
            }
            Expr::Map(a) => return Expr::Map(a),
            Expr::Tree(a) => return Expr::Map(a.into_iter().collect()),
            a => return Expr::error(format!("Invalid expr to-map {}", Expr::from(a))),
        }
    });

    // Convert a list of pairs into a tree.
    env.bind_builtin("to-tree", |env, expr| {
        let a = env.eval(expr[0].clone());
        match a {
            Expr::List(a) => {
                let mut tree = std::collections::BTreeMap::new();
                for e in a {
                    if let Expr::List(e) = e {
                        if e.len() == 2 {
                            tree.insert(e[0].clone(), e[1].clone());
                        } else {
                            return Expr::error(format!("Invalid pair {}", Expr::from(e)));
                        }
                    } else {
                        return Expr::error(format!("Invalid pair {}", Expr::from(e)));
                    }
                }
                Expr::Tree(tree)
            }
            Expr::Map(a) => return Expr::Tree(a.into_iter().collect()),
            Expr::Tree(a) => return Expr::Tree(a),
            a => return Expr::error(format!("Invalid expr to-tree {}", Expr::from(a))),
        }
    });

    env.bind_builtin("to-list", |env, expr| {
        let a = env.eval(expr[0].clone());
        match a {
            Expr::Map(a) => {
                let mut list = vec![];
                for (k, v) in a {
                    list.push(Expr::List(vec![k, v]));
                }
                Expr::List(list)
            }
            Expr::Tree(a) => {
                let mut list = vec![];
                for (k, v) in a {
                    list.push(Expr::List(vec![k, v]));
                }
                Expr::List(list)
            }
            Expr::List(a) => return Expr::List(a),
            a => return Expr::error(format!("Invalid expr to-list {}", a)),
        }
    });

    env.bind_builtin("map", |env, expr| {
        let f = env.eval(expr[0].clone());
        let a = env.eval(expr[1].clone());
        match a {
            Expr::List(a) => {
                let mut list = vec![];
                for e in a {
                    list.push(env.eval(Expr::List(vec![f.clone(), e])));
                }
                Expr::List(list)
            }
            Expr::Map(a) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in a {
                    // map.insert(k.clone(), env.eval(Expr::List(vec![f.clone(), k, v])));
                    let pair = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                    if let Expr::List(pair) = pair {
                        map.insert(pair[0].clone(), pair[1].clone());
                    } else {
                        return Expr::error(format!("Invalid pair {}", pair));
                    }
                }
                Expr::Map(map)
            }
            Expr::Tree(a) => {
                let mut tree = std::collections::BTreeMap::new();
                for (k, v) in a {
                    // tree.insert(k.clone(), env.eval(Expr::List(vec![f.clone(), k, v])));
                    let pair = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                    if let Expr::List(pair) = pair {
                        tree.insert(pair[0].clone(), pair[1].clone());
                    } else {
                        return Expr::error(format!("Invalid pair {}", pair));
                    }
                }
                Expr::Tree(tree)
            }
            a => return Expr::error(format!("Invalid expr map {}", a)),
        }
    });

    env.bind_builtin("filter", |env, expr| {
        let f = env.eval(expr[0].clone());
        let a = env.eval(expr[1].clone());

        match a {
            Expr::List(a) => {
                let mut list = vec![];
                for e in a {
                    if env.eval(Expr::List(vec![f.clone(), e.clone()])) == Expr::Bool(true) {
                        list.push(e);
                    }
                }
                Expr::List(list)
            }
            Expr::Map(a) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in a {
                    let x = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                    if x == Expr::Bool(true) {
                        map.insert(k, v);
                    }
                }
                Expr::Map(map)
            }
            Expr::Tree(a) => {
                let mut tree = std::collections::BTreeMap::new();
                for (k, v) in a {
                    let x = env.eval(Expr::List(vec![f.clone(), k.quote(), v.quote()]));
                    if x == Expr::Bool(true) {
                        tree.insert(k, v);
                    }
                }
                Expr::Tree(tree)
            }
            a => return Expr::error(format!("Invalid expr filter {}", a)),
        }
    });

    env.bind_builtin("reduce", |env, expr| {
        let f = env.eval(expr[0].clone());
        let a = env.eval(expr[1].clone());
        let b = env.eval(expr[2].clone());

        match a {
            Expr::List(a) => {
                let mut acc = b;
                for e in a {
                    acc = env.eval(Expr::List(vec![f.clone(), acc, e]));
                }
                acc
            }
            Expr::Map(a) => {
                let mut acc = b;
                for (k, v) in a {
                    acc = env.eval(Expr::List(vec![f.clone(), acc, k, v]));
                }
                acc
            }
            Expr::Tree(a) => {
                let mut acc = b;
                for (k, v) in a {
                    acc = env.eval(Expr::List(vec![f.clone(), acc, k, v]));
                }
                acc
            }
            a => return Expr::error(format!("Invalid expr reduce {}", a)),
        }
    });

    env.bind_builtin("range", |env, expr| {
        let a = env.eval(expr[0].clone());
        let b = env.eval(expr[1].clone());
        // Check if there is a step
        let c = if expr.len() == 3 {
            env.eval(expr[2].clone())
        } else {
            Expr::Int(1)
        };

        let (a, b) = match (a, b) {
            (Expr::Int(a), Expr::Int(b)) => (a, b),
            (Expr::Float(a), Expr::Float(b)) => (a as i64, b as i64),
            (Expr::Int(a), Expr::Float(b)) => (a, b as i64),
            (Expr::Float(a), Expr::Int(b)) => (a as i64, b),
            (a, b) => return Expr::error(format!("Invalid expr range {} {}", a, b)),
        };

        let c = match c {
            Expr::Int(c) => c,
            Expr::Float(c) => c as i64,
            c => return Expr::error(format!("Invalid expr range {}", c)),
        };

        let mut list = vec![];
        let mut i = a;
        while i <= b {
            list.push(Expr::Int(i));
            i += c;
        }
        Expr::List(list)
    });

    env.bind_builtin("rev", |env, expr| {
        let a = env.eval(expr[0].clone());
        match a {
            Expr::List(mut a) => {
                a.reverse();
                Expr::List(a)
            }
            a => return Expr::error(format!("Invalid expr rev {}", a)),
        }
    });

    env.bind_builtin("rand", |env, expr| {
        use rand::Rng;
        let low = env.eval(expr[0].clone());
        let high = env.eval(expr[1].clone());
        match (low, high) {
            (Expr::Int(low), Expr::Int(high)) => {
                let mut rng = rand::thread_rng();
                Expr::Int(rng.gen_range(low..=high))
            }
            (Expr::Float(low), Expr::Float(high)) => {
                let mut rng = rand::thread_rng();
                Expr::Float(rng.gen_range(low..=high))
            }
            (Expr::Int(low), Expr::Float(high)) => {
                let mut rng = rand::thread_rng();
                Expr::Float(rng.gen_range(low as f64..=high))
            }
            (Expr::Float(low), Expr::Int(high)) => {
                let mut rng = rand::thread_rng();
                Expr::Float(rng.gen_range(low..=high as f64))
            }
            (a, b) => return Expr::error(format!("Invalid expr rand {} {}", a, b)),
        }
    });

    env.bind_builtin("read", |env, expr| {
        // Read a file
        let path = env.eval(expr[0].clone());

        match path {
            Expr::String(path) => {
                let path = std::path::Path::new(&path);
                let file = std::fs::File::open(path).unwrap();
                let reader = std::io::BufReader::new(file);
                let mut code = String::new();
                for line in reader.lines() {
                    code.push_str(&line.unwrap());
                    code.push_str("\n");
                }
                Expr::String(code)
            }
            a => return Expr::error(format!("Invalid expr read {}", a)),
        }
    });

    env.bind_builtin("write", |env, expr| {
        // Write a file
        use std::io::Write;
        let path = env.eval(expr[0].clone());

        let content = env.eval(expr[1].clone());

        match (path, content) {
            (Expr::String(path), Expr::String(content)) => {
                let path = std::path::Path::new(&path);
                let mut file = std::fs::File::create(path).unwrap();
                file.write_all(content.as_bytes()).unwrap();
                Expr::None
            }
            (a, b) => return Expr::error(format!("Invalid expr write {} {}", a, b)),
        }
    });

    env.bind_builtin("shell", |env, expr| {
        // Run a shell command
        let cmd = env.eval(expr[0].clone());

        match cmd {
            Expr::String(cmd) => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .expect("failed to execute process");
                let stdout = String::from_utf8(output.stdout).unwrap();
                let stderr = String::from_utf8(output.stderr).unwrap();
                Expr::List(vec![Expr::String(stdout), Expr::String(stderr)])
            }
            a => return Expr::error(format!("Invalid expr shell {}", a)),
        }
    });

    env
}

pub fn load(mut program: String) {
    let mut env = make_env();
    //let mut program = "(+ 2 2)";

    match Expr::parse(&mut program) {
        Ok(e) => {
            let result = env.eval(e);
            if result != Expr::None {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}

async fn process_cmd(env: &mut Env, mut line: &str) {
    match Expr::parse(&mut line) {
        Ok(e) => {
            let result = env.eval(e);
            if result != Expr::None {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}

pub async fn repl() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    let mut env = make_env();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let prompt = format!("{}> ", "tc".cyan());

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let _ = process_cmd(&mut env, &line).await;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}
