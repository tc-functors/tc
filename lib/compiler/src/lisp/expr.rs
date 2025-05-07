use serde::Serialize;
use serde::de::DeserializeOwned;
use super::symbol::Symbol;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};


use super::env::Env;
use super::builtin::Builtin;
use super::parser;

use nom::{
    error::{convert_error, VerboseError}, Err
};


/// A lisp expression to be evaluated
#[derive(Debug, Default, Clone)]
pub enum Expr {
    /// The unit value (nil)
    #[default]
    None,

    /// A floating point number
    Float(f64),
    /// A signed 64-bit integer
    Int(i64),
    /// A string
    String(String),
    /// A symbol
    Symbol(Symbol),
    /// A boolean
    Bool(bool),

    /// A list of expressions.
    ///
    /// When evaluated, this is used to represent function calls, where the first element
    /// is the function to call, and the rest of the elements are the arguments.
    ///
    /// When used as a data structure, this is used to represent a list of values.
    /// It can be indexed by position, and can be used to store a sequence of values.
    ///
    /// This is stored as a vector, not a linked list, so that it can be indexed efficiently.
    List(Vec<Expr>),
    /// An ordered map of expressions.
    ///
    /// This is helpful when the user desires the ordered properties of a BTreeMap,
    /// such as ordering by key, but with worse time complexities than a HashMap.
    Tree(BTreeMap<Expr, Expr>),
    /// A map of expressions.
    ///
    /// This is helpful when the user wants the time complexities associated with
    /// a HashMap, such as O(1) for insertion, deletion, and lookup, but with no ordering.
    Map(HashMap<Expr, Expr>),

    /// A block of expressions to be evaluated in order.
    ///
    /// This is used to group multiple expressions together, and to evaluate them in sequence.
    /// The result of the block is the result of the last expression in the block.
    ///
    /// This is useful for defining functions, where you want to evaluate multiple expressions
    /// in order, and return the result of the last expression as the result of the function.
    Many(Arc<Vec<Expr>>),

    /// A quoted expression.
    ///
    /// This allows for lazy evaluation of expressions: when a quoted expression is evaluated,
    /// it returns the expression itself, without evaluating it. This is useful for defining
    /// special forms, or for returning unevaluated expressions from functions, like symbols.
    Quote(Box<Expr>),
    /// An error.
    ///
    /// When an error occurs during evaluation, this is used to wrap an error value
    /// that is propagated up the call stack. This allows for error handling in the interpreter.
    Err(Box<Self>),

    /// A function closure.
    ///
    /// This is used to represent a function that takes arguments and returns a value.
    /// The function is defined by a list of arguments, and a body expression to evaluate.
    ///
    /// Internally, the function also keeps track of the environment in which it was defined,
    /// which allows it to capture bindings to variables defined outside the function.
    Function(Option<Box<Env>>, Vec<Expr>, Box<Expr>),
    /// A builtin function.
    ///
    /// This is used to represent a function that is defined in Rust, and can be called from lisp.
    Builtin(Builtin),
}

/// Convert a String to an Expr conveniently.
///
/// This will return a Lisp expression that represents the string, not a symbol.
impl From<String> for Expr {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
/// Convert a &str to an Expr conveniently.
///
/// This will return a Lisp expression that represents the string, not a symbol.
impl From<&str> for Expr {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

/// Convert an i64 to an Expr conveniently.
///
/// This will return a Lisp expression that represents the integer.
impl From<i64> for Expr {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

/// Convert an f64 to an Expr conveniently.
///
/// This will return a Lisp expression that represents the floating point number.
impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

/// Convert a list of expressions to an Expr conveniently.
///
/// This will return a Lisp expression that represents the list of expressions.
impl<T> From<Vec<T>> for Expr
where
    T: Into<Expr>,
{
    fn from(v: Vec<T>) -> Self {
        Self::List(v.into_iter().map(|e| e.into()).collect())
    }
}

/// Allow Serde to convert a serde_json::Value to an Expr.
///
/// This is a convenience method for converting JSON data into Lisp expressions,
/// which will allow us to connect the interpreter to other systems that can serialize
/// data into JSON, and then deserialize it into Lisp expressions.
impl From<serde_json::Value> for Expr {
    fn from(value: serde_json::Value) -> Self {
        use serde_json::Value::*;
        match value.clone() {
            Null => Expr::None,
            Bool(b) => Expr::Bool(b),
            Number(n) => {
                if n.is_f64() {
                    Expr::Float(n.as_f64().unwrap())
                } else {
                    Expr::Int(n.as_i64().unwrap())
                }
            }
            String(s) => Expr::String(s),
            Array(a) => Expr::List(a.into_iter().map(|e| e.into()).collect()),
            Object(o) => Expr::Tree(
                o.into_iter()
                    .map(|(k, v)| (Expr::String(k), v.into()))
                    .collect(),
            ),
        }
    }
}

/// Allow Serde to convert an Expr to a serde_json::Value.
///
/// This is a convenience method for converting Lisp expressions into JSON data,
/// which will allow us to connect the interpreter to other systems that can deserialize
/// JSON data into Lisp expressions, and then serialize it into JSON data.
impl From<Expr> for serde_json::Value {
    fn from(expr: Expr) -> Self {
        use serde_json::Value::*;
        match expr.clone() {
            Expr::None => Null,
            Expr::Bool(b) => Bool(b),
            Expr::Float(f) => Number(serde_json::Number::from_f64(f).unwrap()),
            Expr::Int(i) => Number(serde_json::Number::from(i)),
            Expr::String(s) => String(s),
            Expr::List(l) => Array(l.into_iter().map(|e| e.into()).collect()),
            Expr::Tree(m) => Object(
                m.into_iter()
                    .map(|(k, v)| match (k.into(), v.into()) {
                        (String(k), v) => (k, v),
                        (k, v) => (k.to_string(), v),
                    })
                    .collect(),
            ),
            Expr::Map(m) => Object(
                m.into_iter()
                    .map(|(k, v)| match (k.into(), v.into()) {
                        (String(k), v) => (k, v),
                        (k, v) => (k.to_string(), v),
                    })
                    .collect(),
            ),
            _ => Null,
        }
    }
}


impl Expr {
    /// Serialize a value into a Lisp expression.
    #[inline]
    pub fn serialize<T: Serialize>(x: T) -> Self {
        serde_json::to_value(&x).unwrap().into()
    }

    /// Deserialize a Lisp expression into a value.
    #[inline]
    pub fn deserialize<T: DeserializeOwned>(x: &Self) -> Result<T, serde_json::Error> {
        serde_json::from_value::<T>(x.clone().into())
    }

    /// Create a symbol Lisp expression from a string.
    #[inline]
    pub fn symbol(name: impl ToString) -> Self {
        Self::Symbol(Symbol::new(&name.to_string()))
    }

    /// Wrap another expression in an error value.
    ///
    /// This is useful for propagating errors up the call stack, and for handling errors in the interpreter.
    #[inline]
    pub fn error(message: impl Into<Self>) -> Self {
        Self::Err(Box::new(message.into()))
    }

    /// Quote an expression to prevent it from being evaluated.
    #[inline]
    pub fn quote(&self) -> Self {
        Self::Quote(Box::new(self.clone()))
    }

    /// Apply a callable value to a list of arguments.
    #[inline]
    pub fn apply(&self, args: &[Self]) -> Self {
        let mut result = vec![self.clone()];
        result.extend(args.to_vec());
        Self::List(result)
    }

    /// Parse a string into a Lisp expression.
    ///
    /// If the string is a valid Lisp expression, it will return the parsed expression.
    /// If the string is not a valid Lisp expression, it will return an error message.
    pub fn parse(input: &str) -> Result<Expr, String> {
        let input = Self::remove_comments(input);
        let result = parser::parse_program::<VerboseError<&str>>(input.trim())
            .map(|(_, expr)| expr)
            .map_err(|e| match e {
                Err::Error(e) | Err::Failure(e) => convert_error::<&str>(&input, e),
                Err::Incomplete(e) => unreachable!("Incomplete: {:?}", e),
            });
        result
    }

    /// Strip the comments from an input string.
    ///
    /// This is used to remove comments from a string before parsing it.
    fn remove_comments(input: &str) -> String {
        let mut input = input;
        let mut output = String::new();
        while !input.is_empty() {
            // Ignore comments in quoted strings

            if input.starts_with('"') {
                let mut last_was_escape = false;
                let mut len = 0;
                for c in input[1..].chars() {
                    len += 1;
                    if c == '\\' && !last_was_escape {
                        last_was_escape = true;
                        continue;
                    }
                    if last_was_escape {
                        last_was_escape = false;
                        continue;
                    }

                    if c == '"' {
                        break;
                    }
                }

                output.push_str(&input[..len + 1]);
                input = &input[len + 1..];
            }

            if input.starts_with(';') {
                let end = input.find('\n').unwrap_or(input.len());
                input = &input[end..];
            } else if !input.is_empty() {
                output.push_str(&input[..1]);
                input = &input[1..];
            }
        }
        output
    }
}

/// Compare two expressions for equality.
///
/// This allows you to compare two expressions using the `==` operator.
impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        use Expr::*;
        match (self, other) {
            (None, None) => true,
            (Builtin(f1), Builtin(f2)) => f1 as *const _ == f2 as *const _,
            (Float(f1), Float(f2)) => f1.to_bits() == f2.to_bits(),
            (Int(i1), Int(i2)) => i1 == i2,
            (Int(i), Float(f)) | (Float(f), Int(i)) => *f == *i as f64,
            (String(s1), String(s2)) => s1 == s2,
            (Symbol(s1), Symbol(s2)) => s1 == s2,
            (List(l1), List(l2)) => l1 == l2,
            (Tree(t1), Tree(t2)) => t1 == t2,
            (Map(m1), Map(m2)) => m1 == m2,
            (Function(_, args1, body1), Function(_, args2, body2)) => {
                args1 == args2 && body1 == body2
            }
            (Quote(e1), Quote(e2)) => e1 == e2,
            (Err(e1), Err(e2)) => e1 == e2,
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Many(d1), Many(d2)) => d1 == d2,
            _ => false,
        }
    }
}

/// Compare two expressions for ordering.
///
/// This allows you to compare two expressions using the `<`, `>`, `<=`, and `>=` operators,
/// as well as to sort expressions in a collection.
impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Expr::*;
        match (self, other) {
            (None, None) => Some(std::cmp::Ordering::Equal),
            (Float(f1), Float(f2)) => f1.partial_cmp(f2),
            (Int(i1), Int(i2)) => i1.partial_cmp(i2),
            (Int(i), Float(f)) => (*i as f64).partial_cmp(f),
            (Float(f), Int(i)) => f.partial_cmp(&(*i as f64)),
            (String(s1), String(s2)) => s1.partial_cmp(s2),
            (Symbol(s1), Symbol(s2)) => s1.partial_cmp(s2),
            (List(l1), List(l2)) => l1.partial_cmp(l2),
            (Tree(t1), Tree(t2)) => t1.partial_cmp(t2),
            (Map(m1), Map(m2)) => {
                let m1 = BTreeMap::from_iter(m1.iter());
                let m2 = BTreeMap::from_iter(m2.iter());
                m1.partial_cmp(&m2)
            }
            (Quote(e1), Quote(e2)) => e1.partial_cmp(e2),
            (Function(_, args1, body1), Function(_, args2, body2)) => {
                if args1 == args2 {
                    body1.partial_cmp(body2)
                } else {
                    args1.partial_cmp(args2)
                }
            }
            (Err(e1), Err(e2)) => e1.partial_cmp(e2),
            (Builtin(f1), Builtin(f2)) => {
                (f1 as *const _ as usize).partial_cmp(&(f2 as *const _ as usize))
            }
            (Bool(b1), Bool(b2)) => b1.partial_cmp(b2),
            (Many(d1), Many(d2)) => d1.partial_cmp(d2),
            _ => Option::None,
        }
    }
}

/// Compare two expressions for strong equality, where a == b and b == a.
impl Eq for Expr {}

/// Compare two expressions for strong ordering, where a < b and b > a.
impl Ord for Expr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Less)
    }
}

/// Hash an expression for use in a hash map or set.
///
/// This allows you to use expressions as keys in a hash map or set, which is useful
/// for storing data in a way that allows for fast lookups and comparisons.
impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Expr::*;
        // Write the tag as an integer to the hasher
        state.write_u8(match self {
            None => 0,
            Float(_) => 1,
            Int(_) => 2,
            Bool(_) => 3,
            String(_) => 4,
            Symbol(_) => 5,
            List(_) => 6,
            Tree(_) => 7,
            Map(_) => 8,
            Many(_) => 9,
            Quote(_) => 10,
            Err(_) => 11,
            Function(_, _, _) => 12,
            Builtin(_) => 13,
        });

        match self {
            None => 0.hash(state),
            Float(f) => f.to_bits().hash(state),
            Int(i) => i.hash(state),
            Bool(b) => b.hash(state),
            String(s) => s.hash(state),
            Symbol(s) => s.hash(state),
            List(l) => l.hash(state),
            Tree(t) => t.hash(state),
            Map(m) => BTreeMap::from_iter(m.iter()).hash(state),
            Many(d) => d.hash(state),
            Quote(e) => e.hash(state),
            Err(e) => e.hash(state),
            Function(_, args, body) => {
                args.hash(state);
                body.hash(state);
            }
            Builtin(f) => (f as *const _ as usize).hash(state),
        }
    }
}

/// Implement display for Lisp expressions.
///
/// This allows you to print a Lisp expression as a string, which is useful for debugging.
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use Expr::*;
        match self {
            None => write!(f, "nil"),
            Float(n) => write!(f, "{}", n),
            Int(n) => write!(f, "{}", n),
            Bool(b) => write!(f, "{}", b),
            String(s) => write!(f, "\"{}\"", s),
            Symbol(s) => write!(f, "{}", s.name()),
            Quote(e) => write!(f, "'{}", e),
            Err(e) => write!(f, "<error: {}>", e),
            Many(d) => {
                write!(f, "{{ ")?;
                for (i, e) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, " }}")
            }
            List(e) => {
                write!(f, "(")?;
                for (i, e) in e.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Tree(t) => {
                write!(f, "[")?;
                for (i, (k, v)) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "]")
            }
            Map(m) => {
                write!(f, "#[")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "]")
            }
            Function(_, args, body) => {
                write!(f, "(lambda (")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") {})", body)
            }
            Builtin(b) => write!(f, "<builtin {}>", b.name),
        }
    }
}
