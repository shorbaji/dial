use std::fmt::Display;

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
    Pair(Vec<Object>),
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
    
    fn list(mut v: Vec<Object>) -> Object {
        Object::Pair(v)
    }
    
    fn write(&self) -> Result<String, &'static str> {
        match self {
            Object::Boolean(b) => Ok(format!("{}", b)),
            Object::Char(c) => Ok(format!("{}", c)),
            Object::Eof => Ok(format!("eof")),
            Object::Null => Ok(format!("null")),
            Object::Number(n) => Ok(format!("{}", n)),
            Object::Pair(v) => Ok(format!(
                "({} . {})",
                v[0].write()?,
                v[1].write()?,
            )),
            Object::String(s) => Ok(format!("\"{}\"", s)),
            Object::Symbol(s) => Ok(format!("{}", s)),
            Object::Procedure(p) => Ok(format!("proc")),
            Object::Unspecified => Ok(format!("<unspecified>")),
            _ => Err("don't know how to represent this"),
        }
        
    }
    
    fn car(&self) -> Result<&Object, &'static str> {
        match self {
            Object::Pair(v) => Ok(&v[0]),
            _ => Err("not a pair"),
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
    parent: Option<Box<Env>>,
    hashmap: std::collections::HashMap<String, Object>,
}

impl Env {
    fn new() -> Self {
        Self {
            parent: None,
            hashmap: std::collections::HashMap::new(),    
        }
    }
    
    fn insert(&mut self, location:&str, object: Object) -> Option<Object> {
        self.hashmap.insert(location.to_string(), object)
    }
    
    fn lookup(&self, location: &str) -> Option<&Object> {
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
struct Procedure {
    func : fn (Vec<Object>) -> Result<Object, &'static str>,
}

impl Procedure {
    fn apply<'a>(&'a self, args: Vec<Object>) -> Result<Object, &'static str>{
        (self.func)(args)
    }
}

fn add<'a>(v: Vec<Object>) -> Result<Object, &'static str> {
    
    let mut acc: Number = 0;
    
    for object in v {
        match object {
            Object::Number(n) => {
                acc += n;
            },
            _ => {return Err("not a number");}
        }
    };
    
    Ok(Object::Number(acc))
}

fn evlis<'a, 'b: 'a>(exprs: &'a [Object], env: &'a mut Env) -> Result<Vec<Object>, &'static str> {
    exprs
    .into_iter()
    .map(|exp| eval(exp, env))
    .collect()
}

fn eval<'a>(expr: &'a Object, env: &'a mut Env) -> Result<Object, &'static str> {
    match expr {
        Object::Symbol(s) => match env.lookup(s) {
            Some(object) => Ok(object.clone()),
            None => Err("symbol not found"),
        },
        Object::Boolean(_) |
        Object::ByteVector |
        Object::Char(_) |
        Object::Null |
        Object::Number(_) |
        Object::String(_) |
        Object::Vector (_)=> Ok(expr.clone()),
        Object::Pair(v) => match v[0] {
            Object::Keyword("if") => Ok(expr.clone()),
            Object::Keyword("quote") => Ok(v[1].clone()),
            _ => {
                let operator = &v[0];
                let operands = &v[1..];
                
                match eval(&v[0], env)? {
                    Object::Procedure(procedure) => {
                        Ok(procedure.apply(
                            evlis(&v[1..],env)?)?)},
                            _ =>  Err("not a proc"),
                        }
                    },
            },
        _ => return Err("can't eval this"),  
    }
}
        
        
        fn main() {
            let mut env = Env::new();
            env.insert("x", Object::Boolean(true));
            env.insert("+", Object::Procedure(Procedure {func: add}));
            
            let exprs= vec!(
                Object::Number(1),
                Object::Null,
                Object::String("hello, world!".to_string()),
                Object::Symbol("x".to_string()),
                Object::Symbol("+".to_string()),
                Object::Pair(vec!(
                    Object::Keyword("quote"),
                    Object::Symbol("x".to_string()),    
                )),
                Object::Pair(vec!(
                    Object::Keyword("quote"),
                    Object::Pair(vec![
                        Object::Symbol("x".to_string()),
                        Object::Number(1),
                        ]),    
                    )),
                    Object::Pair(vec!(
                        Object::Symbol("+".to_string()),
                        Object::Number(1),
                        Object::Number(2),
                    )),
                );
                
                let mut object = Object::Null;
                
                for expr in exprs {
                    println!("{}", eval(&expr, &mut env).unwrap());
                }
            }
            