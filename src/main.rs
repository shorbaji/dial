use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Display,
    rc::{Rc, Weak},
};

//
// Object
//
#[derive(Clone)]
enum Object {
    Boolean(bool),
    ByteVector,
    Char(char),
    Eof,
    Keyword(&'static str),
    Null,
    Number(Number),
    Pair(Rc<Object>, Rc<Object>),
    Port,
    Procedure(Procedure),
    String(String),
    Symbol(String),
    Unspecified,
    Vector(Vec<Object>),
}

impl Object {
    fn cons(car: Rc<Object>, cdr: Rc<Object>) -> Rc<Self> {
        Rc::new(Self::Pair(car, cdr))
    }

    fn from_vec(v: &[Object]) -> Rc<Self> {
        if v.is_empty() {
            Rc::new(Self::Null)
        } else {
            let car = v[0].clone();
            let cdr = &v[1..];

            Self::cons(Rc::new(car), Self::from_vec(cdr))
        }
    }

    fn write(&self) -> Result<String, &'static str> {
        match self {
            Object::Boolean(b) => Ok(format!("{}", b)),
            Object::Char(c) => Ok(format!("{}", c)),
            Object::Eof => Ok(format!("eof")),
            Object::Null => Ok(format!("null")),
            Object::Number(n) => Ok(format!("{}", n)),
            Object::Pair(a, b) => Ok(format!("({} . {})", a.write()?, b.write()?,)),
            Object::String(s) => Ok(format!("\"{}\"", s)),
            Object::Symbol(s) => Ok(format!("{}", s)),
            Object::Procedure(_) => Ok(format!("proc")),
            Object::Unspecified => Ok(format!("<unspecified>")),
            Object::Keyword(k) => Ok(format!("{}", k)),
            _ => Err("don't know how to represent this"),
        }
    }

    fn car(&self) -> Result<&Object, &'static str> {
        match self {
            Object::Pair(a, _) => Ok(a),
            _ => Err("not a pair"),
        }
    }

    fn symbol_to_str(&self) -> Result<String, &'static str> {
        match self {
            Object::Symbol(s) => Ok(s.clone()),
            _ => Err("not a symbol"),
        }
    }

    fn apply(&self, operands: Rc<Object>) -> Result<Rc<Object>, &'static str> {
        let operands = operands.to_vec()?;

        match self {
            Object::Procedure(proc) => proc.call(operands),
            _ => Err("not a proc"),
        }
    }

    fn to_vec(&self) -> Result<VecDeque<Rc<Object>>, &'static str> {
        match self {
            Object::Pair(car, cdr) => {
                let mut rest = cdr.to_vec()?;
                let mut v = VecDeque::new();
                v.push_front(car.clone());
                v.append(&mut rest);
                Ok(v)
            }
            Object::Null => Ok(VecDeque::new()),
            _ => Err("malformed evlis"),
        }
    }

    fn evlis(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => Ok(Object::cons(
                car.clone().eval(envr.clone())?,
                cdr.evlis(envr)?,
            )),
            _ => Ok(Rc::new(Object::Null)),
        }
    }

    fn is_true(&self) -> bool {
        match self {
            Object::Null => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }

    fn eval_antecedent(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(antecedent, _cdr) => antecedent.clone().eval(envr),
            _ => Err("malformed consequent in if statement"),
        }
    }

    fn eval_if(&self, flag: bool, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(consequent, cdr) => {
                if flag {
                    consequent.clone().eval(envr)
                } else {
                    cdr.eval_antecedent(envr)
                }
            }
            _ => Err("malformed consequent in if statement"),
        }
    }

    fn ifify(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => {
                let predicate = car.clone();

                cdr.eval_if(predicate.eval(envr.clone())?.is_true(), envr)
            }
            _ => Err("malformed if statement"),
        }
    }

    fn self_eval(&self) -> Rc<Object> {
        Rc::new(self.clone())
    }

    fn eval_lambda(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => Ok(Rc::new(Object::Procedure(Procedure::Lambda(
                car.clone(),
                cdr.clone(),
                Rc::downgrade(&envr),
            )))),
            _ => Err("malformed lambda expression"),
        }
    }

    fn eval_define(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => {
                match &*car.clone() {
                    Object::Symbol(s) => {
                        envr.borrow_mut().insert(&s, cdr.car()?.eval(envr.clone())?);
                        Ok(Rc::new(Object::Unspecified))
                    },
                    _ => Err("malformed define expression - need a symbol")
                }
            },
            _ => Err("malformed define expression - no value"),
        }
    }
    fn eval(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Symbol(s) => match envr.borrow().lookup(&s) {
                Some(object) => Ok(object.clone()),
                None => Err("symbol not found"),
            },
            Object::Boolean(_)
            | Object::ByteVector
            | Object::Char(_)
            | Object::Null
            | Object::Number(_)
            | Object::String(_)
            | Object::Vector(_) => Ok(self.self_eval()),
            Object::Pair(car, cdr) => {
                let car = car.clone();

                match *car {
                    Object::Keyword("if") => cdr.ifify(envr),
                    Object::Keyword("quote") => Ok(cdr.clone()),
                    Object::Keyword("lambda") => cdr.eval_lambda(envr),
                    Object::Keyword("define") => cdr.eval_define(envr),
                    _ => car.eval(envr.clone())?.apply(cdr.evlis(envr)?),
                }
            }
            _ => return Err("can't eval this"),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.write().unwrap())
    }
}
//
// Env
//
//

struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    hashmap: std::collections::HashMap<String, Rc<Object>>,
}

impl Env {
    fn new() -> Self {
        Self {
            parent: None,
            hashmap: std::collections::HashMap::new(),
        }
    }

    fn insert(&mut self, location: &str, object: Rc<Object>) -> Option<Rc<Object>> {
        self.hashmap.insert(location.to_string(), object)
    }

    fn lookup(&self, symbol: &str) -> Option<Rc<Object>> {
        match self.hashmap.get(symbol) {
            Some(r) => Some(r.clone()),
            None => self
                .parent
                .as_deref()
                .and_then(|r| r.borrow().lookup(symbol)),
        }
    }
}
//
// Miscellaneous
//
//
type Number = i128;

//
// Procedure
//
//

#[derive(Clone)]
enum Procedure {
    Builtin(fn(v: VecDeque<Rc<Object>>) -> Result<Rc<Object>, &'static str>),
    Lambda(Rc<Object>, Rc<Object>, Weak<RefCell<Env>>),
}

impl Procedure {
    fn call(&self, args: VecDeque<Rc<Object>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Procedure::Builtin(func) => func(args),
            Procedure::Lambda(vars, body, envr) => {
                let child_envr = Env {
                    parent: envr.upgrade(),
                    hashmap: HashMap::new(),
                };

                let child_envr = Rc::new(RefCell::new(child_envr));

                let vars = vars.to_vec()?;

                if vars.len() == args.len() {
                    let mut last: Rc<Object> = Rc::new(Object::Unspecified);

                    for (var, arg) in vars.iter().zip(args.iter()) {
                        child_envr
                            .borrow_mut()
                            .insert(&var.symbol_to_str()?, arg.clone());
                    }

                    for expr in body.to_vec()? {
                        last = expr.eval(child_envr.clone())?
                    }

                    Ok(last)
                } else {
                    Err("wrong number of args")
                }
            }
        }
    }
}

fn add<'a>(objects: VecDeque<Rc<Object>>) -> Result<Rc<Object>, &'static str> {
    let mut acc: Number = 0;

    for object in objects.iter() {
        match **object {
            Object::Number(n) => {
                acc += n;
            }
            _ => {
                return Err("not a number");
            }
        }
    }

    Ok(Rc::new(Object::Number(acc)))
}

fn read(code: &str) -> Result<Rc<Object>, &'static str> {
    let tokens = tokenize(code.to_string());
    Ok(Rc::new(parse(false, &tokens[..])?))
}

fn tokenize(code: String) -> Vec<String> {
    code.replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

fn grab(tokens: &[String]) -> &[String] {
    let start = 0;
    let mut end = 0;

    if tokens[start] == "(" {
        let mut count = 1;
        end += 1;
        while count > 0 {
            if tokens[end] == "(" {
                count += 1;
            } else if tokens[end] == ")" {
                count -= 1;
            }
            end += 1;
        }
    }
    &tokens[start..end]
}

fn parse_atom(s: &str) -> Result<Object, &'static str> {
    match s {
        "lambda" => Ok(Object::Keyword("lambda")),
        "if" => Ok(Object::Keyword("if")),
        "quote" => Ok(Object::Keyword("quote")),
        "define" => Ok(Object::Keyword("define")),
        _ => match s.parse::<i128>() {
            Ok(n) => Ok(Object::Number(n)),
            Err(_) => Ok(Object::Symbol(s.to_string())),
        },
    }
}

fn parse(open: bool, tokens: &[String]) -> Result<Object, &'static str> {
    if open {
        if tokens[0] == "(" {
            if tokens.len() == 1 {
                Err("unclosed parenthesis")
            } else {
                // let g = grab(tokens)?;
                Ok(Object::Pair(
                    Rc::new(parse(false, tokens)?),
                    Rc::new(parse(true, &tokens[grab(tokens).len()..])?),
                ))
            }
        } else if tokens[0] == ")" {
            Ok(Object::Null)
        } else {
            if tokens.len() == 1 {
                Err("unclosed parenthesis")
            } else {
                let car = parse_atom(&tokens[0])?;

                Ok(Object::Pair(
                    Rc::new(car),
                    Rc::new(parse(open, &tokens[1..])?),
                ))
            }
        }
    } else {
        if tokens[0] == "(" {
            if tokens.len() == 1 {
                Err("unclosed parenthesis")
            } else {
                parse(true, &tokens[1..])
            }
        } else if tokens[0] == ")" {
            Err("unexpected close bracket")
        } else {
            match tokens[0].parse::<i128>() {
                Ok(n) => Ok(Object::Number(n)),
                Err(_) => Ok(Object::Symbol(tokens[0].clone())),
            }
        }
    }
}
fn main() {
    let mut envr = Env::new();

    envr.insert("+", Rc::new(Object::Procedure(Procedure::Builtin(add))));

    let envr = Rc::new(RefCell::new(envr));

    let codes = vec![
        "1",
        "(+ 1 2)",
        "((lambda (x) (+ x x)) 42)",
        "()",
        "(+ 1 2)",
        "(quote 1 2 3 4)",
        "(define x 1)",
        "x",
        "(define double (lambda (x) (+ x x)))",
        "(double 2)",
        "(double (double 4))"
    ];

    for code in codes {
        let expr = read(code).unwrap();
        print!("{} => ", code);
        println!("{}", expr.eval(envr.clone()).unwrap());
    }
}
