use std::{fmt::Display, rc::{Rc, Weak}, path::Iter, collections::VecDeque, cell::{RefCell, Ref}};

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
    fn new() -> Self {
        Self::Unspecified
    }
    
    fn from_vec(v: &[Object]) -> Rc<Self> {
        if v.is_empty() {
            Rc::new(Object::Null)
        } else {
            let car = v[0].clone();
            let cdr = &v[1..];
        
            Object::cons(Rc::new(car), Object::from_vec(cdr))    
        }
    }
    
    fn write(&self) -> Result<String, &'static str> {
        match self {
            Object::Boolean(b) => Ok(format!("{}", b)),
            Object::Char(c) => Ok(format!("{}", c)),
            Object::Eof => Ok(format!("eof")),
            Object::Null => Ok(format!("null")),
            Object::Number(n) => Ok(format!("{}", n)),
            Object::Pair(a, b) => Ok(format!(
                "({} . {})",
                a.write()?,
                b.write()?,
            )),
            Object::String(s) => Ok(format!("\"{}\"", s)),
            Object::Symbol(s) => Ok(format!("{}", s)),
            Object::Procedure(_) => Ok(format!("proc")),
            Object::Unspecified => Ok(format!("<unspecified>")),
            Object::Keyword(k) => Ok(format!("{}", k)),
            _ => Err("don't know how to represent this"),
        }
        
    }
    
    fn cons(car: Rc<Object>, cdr: Rc<Object>) -> Rc<Object> {
        Rc::new(Object::Pair(car, cdr))
    }
    
    fn car(&self) -> Result<&Object, &'static str> {
        match self {
            Object::Pair(a, _) => Ok(a),
            _ => Err("not a pair"),
        }
    }

    fn apply(&self, operands: Rc<Object>) -> Result<Rc<Object>, &'static str>{
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
            },
            Object::Null => {
                Ok(VecDeque::new())
            },
            _ => Err("malformed evlis")
        }
    }

    fn evlis(&self, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => { 
                Ok(Object::cons(car.clone().eval(envr.clone())?, cdr.evlis(envr)?))
            },
            _ => Ok(Rc::new(Object::Null))
        } 
    }

    fn is_true(&self) -> bool {
        match self {
            Object::Null => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }


    fn eval_antecedent(&self, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(antecedent, cdr) => antecedent.clone().eval(envr),
            _ => Err("malformed consequent in if statement"),
        }
    }

    fn eval_if(&self, flag: bool, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str>{
        
        match self {
            Object::Pair(consequent, cdr) => {
                if flag {
                    consequent.clone().eval(envr)
                } else {
                    cdr.eval_antecedent(envr)
                }
            },
            _ => Err("malformed consequent in if statement"),
        }
    }

    fn ifify(&self, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => {
                let predicate = car.clone();

                cdr.eval_if(predicate.eval(envr.clone())?.is_true(), envr)
            },
            _ => Err("malformed if statement")
        }
    }

    fn self_eval(&self) -> Rc<Object> {
        Rc::new(self.clone())
    } 

    fn eval_lambda(&self, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Pair(car, cdr) => {
                Ok(Rc::new(Object::Procedure(Procedure::Lambda(car.clone(), cdr.clone(), Rc::downgrade(&envr)))))
            },
            _ => Err("malformed lambda expression")
        }
    }

    fn eval(&self, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Object::Symbol(s) => lookup_symbol(s.clone(), envr),
            Object::Boolean(_) |
            Object::ByteVector |
            Object::Char(_) |
            Object::Null |
            Object::Number(_) |
            Object::String(_) |
            Object::Vector (_) => Ok(self.self_eval()),
            Object::Pair(car, cdr) => {
                let car = car.clone();
    
                match *car {
                    Object::Keyword("if") => cdr.ifify(envr),
                    Object::Keyword("quote") => Ok(cdr.clone()),
                    Object::Keyword("lambda") => cdr.eval_lambda(envr),
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

impl Iterator for Object {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
// 
// Env
// 
// 

struct Env {
    parent: Option<Rc<Env>>,
    hashmap: std::collections::HashMap<String, Rc<Object>>,
}

impl Env {
    fn new() -> Self {
        Self {
            parent: None,
            hashmap: std::collections::HashMap::new(),    
        }
    }
    
    fn insert(&mut self, location:&str, object: Rc<Object>) -> Option<Rc<Object>> {
        self.hashmap.insert(location.to_string(), object)
    }
    
    fn lookup(&self, location: &str) -> Option<&Rc<Object>> {
        self.hashmap.get(location) 
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
    fn call(&self, operands: VecDeque<Rc<Object>>) -> Result<Rc<Object>, &'static str> {
        match self {
            Procedure::Builtin(func) => func(operands),
            Procedure::Lambda(_, _, _) => Err("don't yet know how to call a lambda")
        }
    }
}

fn add<'a>(objects: VecDeque<Rc<Object>>) -> Result<Rc<Object>, &'static str> {
    let mut acc: Number = 0;

    for object in objects.iter() {
        match **object {
            Object::Number(n) => {
                acc += n;
            },
            _ => {return Err("not a number");}
        }
    };
    
    Ok(Rc::new(Object::Number(acc)))
}

fn lookup_symbol(s: String, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
    match envr.borrow().lookup(&s) {
        Some(object) => {
            Ok(object.clone())
        },
        None => Err("symbol not found"),
    }
}


// fn eval_pair(car: Rc<Object>, cdr: Rc<Object>, mut envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, &'static str> {
// }

fn main() {
    let mut envr = Rc::new(RefCell::new(Env::new()));

    envr.borrow_mut().insert("x", Rc::new(Object::Boolean(true)));
    envr.borrow_mut().insert("+", Rc::new(Object::Procedure(Procedure::Builtin(add))));
        
    let v = vec!(Object::Keyword("quote"), Object::Symbol("x".to_string()));
    let quote_expr_atom = Object::from_vec(&v[..]);
    
    let v = vec!(Object::Keyword("quote"), Object::Symbol("x".to_string()), Object::Number(1));
    let quote_expr_pair = Object::from_vec(&v[..]);
    
    let v = vec!( Object::Symbol("+".to_string()), Object::Number(1), Object::Number(2));
    let func_call_expr = Object::from_vec(&v[..]);

    let v = vec!(Object::Keyword("if"), Object::Boolean(false), Object::Number(1), Object::Number(2));
    let if_expr = Object::from_vec(&v[..]);

    let v = vec!(Object::Keyword("lambda"), Object::Boolean(false), Object::Number(1), Object::Number(2));
    let lambda_expr = Object::from_vec(&v[..]);

    let exprs= vec!(
        Rc::new(Object::Number(1)),
        Rc::new(Object::Null),
        Rc::new(Object::String("hello, world!".to_string())),
        Rc::new(Object::Symbol("x".to_string())),
        Rc::new(Object::Symbol("+".to_string())),
        quote_expr_atom,
        quote_expr_pair,
        func_call_expr,
        if_expr,
        lambda_expr);
    

    for expr in exprs {
        print!("{} => ", expr);
        println!("{}", expr.eval(envr.clone()).unwrap());
    }
}
                            