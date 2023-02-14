use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Display,
    io::{self, Write},
    rc::{Rc, Weak},
};

#[derive(Debug)]
struct DialError {
    code: i16,
}

impl DialError {
    fn new(code: i16) -> Self {
        Self {
            code: code,
        }
    }
}
impl Display for DialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self.code {
            100 => "no external representation",
            101 => "bad argument - not a pair",
            102 => "not a symbol",
            103 => "not a proc",
            104 => "malformed evlis",
            105 => "malformed if statement",
            106 => "malformed lambda expression",
            107 => "malformed define expression",
            108 => "symbol not found",
            109 => "can't eval this",
            110 => "wrong number of args",
            111 => "not a number",
            200 => "missing close parenthesis",
            201 => "unexpected close bracket",
            202 => "unexpected end of expr",
            _ => "unknown error",
        };
        write!(f, "error code: {}: {}", self.code, text)

    }
}

//
// Object
//
#[derive(Clone, Debug)]
enum Object {
    Boolean(bool),
    ByteVector,
    Char(char),
    // Eof,
    Keyword(&'static str),
    Null,
    Number(Number),
    Pair(Rc<Object>, Rc<Object>),
    // Port,
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

    fn write(&self) -> Result<String, DialError> {
        match self {
            Object::Boolean(b) => Ok(format!("{}", if *b {"#t"} else {"#f"})),
            Object::Char(c) => Ok(format!("{}", c)),
            // Object::Eof => Ok(format!("eof")),
            Object::Null => Ok(format!("null")),
            Object::Number(n) => Ok(format!("{}", n)),
            Object::Pair(a, b) => Ok(format!("({} . {})", a.write()?, b.write()?,)),
            Object::String(s) => Ok(format!("\"{}\"", s)),
            Object::Symbol(s) => Ok(format!("{}", s)),
            Object::Procedure(_) => Ok(format!("proc")),
            Object::Unspecified => Ok(format!("<unspecified>")),
            Object::Keyword(k) => Ok(format!("{}", k)),
            _ => Err(DialError::new(100)),
        }
    }

    fn car(&self) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Pair(a, _) => Ok(a.clone()),
            _ => Err(DialError::new(101)),
        }
    }

    fn symbol_to_str(&self) -> Result<String, DialError> {
        match self {
            Object::Symbol(s) => Ok(s.clone()),
            _ => Err(DialError::new(102)),
        }
    }

    fn apply(&self, operands: Rc<Object>) -> Result<Rc<Object>, DialError> {
        let operands = operands.to_vec()?;

        match self {
            Object::Procedure(proc) => proc.call(operands),
            _ => Err(DialError::new(103)),
        }
    }

    fn to_vec(&self) -> Result<VecDeque<Rc<Object>>, DialError> {
        match self {
            Object::Pair(car, cdr) => {
                let mut rest = cdr.to_vec()?;
                let mut v = VecDeque::new();
                v.push_front(car.clone());
                v.append(&mut rest);
                Ok(v)
            }
            Object::Null => Ok(VecDeque::new()),
            _ => Err(DialError::new(104)),
        }
    }

    fn evlis(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Pair(car, cdr) => Ok(Object::cons(
                car.eval(envr.clone())?,
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

    fn eval_antecedent(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Pair(antecedent, _cdr) => antecedent.eval(envr),
            _ => Err(DialError { code: 105 }),
        }
    }

    fn eval_if(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Pair(car, cdr) => {
                let predicate = car;
                let predicate = predicate.eval(envr.clone())?.is_true();
                match &**cdr {
                    Object::Pair(car, cdr) => {
                        if predicate {
                            car.eval(envr)
                        } else {
                            cdr.eval_antecedent(envr)
                        }
                    },
                    _ => Err(DialError { code: 105 }),                     
                }
            }
            _ => Err( DialError { code: 105 }),
        }
    }

    fn self_eval(&self) -> Rc<Object> {
        Rc::new(self.clone())
    }

    fn eval_lambda(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Pair(car, cdr) => Ok(Rc::new(Object::Procedure(Procedure::Lambda(
                car.clone(),
                cdr.clone(),
                Rc::downgrade(&envr),
            )))),
            _ => Err(DialError { code: 106 }),
        }
    }

    fn eval_define(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Pair(car, cdr) => {
                match &*car.clone() {
                    Object::Symbol(s) => {
                        envr.borrow_mut().insert(&s, cdr.car()?.eval(envr.clone())?);
                        Ok(Rc::new(Object::Unspecified))
                    },
                    _ => Err(DialError { code: 107 })
                }
            },
            _ => Err(DialError { code: 107 }),
        }
    }
    
    fn eval(&self, envr: Rc<RefCell<Env>>) -> Result<Rc<Object>, DialError> {
        match self {
            Object::Symbol(s) => match envr.borrow().lookup(&s) {
                Some(object) => Ok(object),
                None => Err(DialError { code: 108 }),
            },
            Object::Boolean(_)
            | Object::ByteVector
            | Object::Char(_)
            | Object::Null
            | Object::Number(_)
            | Object::String(_)
            | Object::Vector(_) => Ok(self.self_eval()),
            Object::Pair(car, cdr) => {
                match **car {
                    Object::Keyword("quote") => cdr.clone().car(),
                    Object::Keyword("if") => cdr.eval_if(envr),
                    Object::Keyword("lambda") => cdr.eval_lambda(envr),
                    Object::Keyword("define") => cdr.eval_define(envr),
                    _ => car.eval(envr.clone())?.apply(cdr.evlis(envr)?),
                }
            }
            _ => return Err(DialError { code: 109 }),
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


#[derive(Debug)]
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
            Some(object) => Some(object.clone()),
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

#[derive(Clone, Debug)]
enum Procedure {
    Builtin(fn(v: VecDeque<Rc<Object>>) -> Result<Rc<Object>, DialError>),
    Lambda(Rc<Object>, Rc<Object>, Weak<RefCell<Env>>),
}

impl Procedure {
    fn call(&self, args: VecDeque<Rc<Object>>) -> Result<Rc<Object>, DialError> {
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
                    Err(DialError { code: 110 })
                }
            }
        }
    }
}

fn add<'a>(objects: VecDeque<Rc<Object>>) -> Result<Rc<Object>, DialError> {
    let mut acc: Number = 0;

    for object in objects.iter() {
        match **object {
            Object::Number(n) => {
                acc += n;
            }
            _ => {
                return Err(DialError { code: 111 });
            }
        }
    }

    Ok(Rc::new(Object::Number(acc)))
}


fn create_global_envr() -> Rc<RefCell<Env>> {
    let envr = Rc::new(RefCell::new(Env::new()));

    envr.borrow_mut().insert("+", Rc::new(Object::Procedure(Procedure::Builtin(add))));

    envr
}

struct Reader {
    buffer: String,
}

impl Reader {
    fn read(&self, code: &str) -> Result<Rc<Object>, DialError> {
        let tokens = self.tokenize(code.to_string());
        Ok(Rc::new(self.parse(false, &tokens[..])?))
    }
    
    fn tokenize(&self, code: String) -> Vec<String> {
        code.replace("(", " ( ")
            .replace(")", " ) ")
            .split_whitespace()
            .map(|x| x.to_string())
            .collect()
    }
    
    fn grab<'a>(&self, tokens: &'a [String]) -> &'a [String] {
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
    
    fn parse_atom(&self, s: &str) -> Result<Object, DialError> {
        match s {
            "lambda" => Ok(Object::Keyword("lambda")),
            "if" => Ok(Object::Keyword("if")),
            "quote" => Ok(Object::Keyword("quote")),
            "define" => Ok(Object::Keyword("define")),
            "#t" => Ok(Object::Boolean(true)),
            "#f" => Ok(Object::Boolean(false)),
            _ => match s.parse::<i128>() {
                Ok(n) => Ok(Object::Number(n)),
                Err(_) => Ok(Object::Symbol(s.to_string())),
            },
        }
    }
    
    fn parse(&self, open: bool, tokens: &[String]) -> Result<Object, DialError> {
        if open {
            if tokens.is_empty() {
                Err(DialError { code: 200 })
            } else if tokens[0] == "(" {
                if tokens.len() == 1 {
                    Err(DialError { code: 200 })
                } else {
                    // let g = grab(tokens)?;
                    Ok(Object::Pair(
                        Rc::new(self.parse(false, tokens)?),
                        Rc::new(self.parse(true, &tokens[self.grab(tokens).len()..])?),
                    ))
                }
            } else if tokens[0] == ")" {
                Ok(Object::Null)
            } else {
                if tokens.len() == 1 {
                    Err(DialError { code: 200 })
                } else {
                    let car = self.parse_atom(&tokens[0])?;
    
                    Ok(Object::Pair(
                        Rc::new(car),
                        Rc::new(self.parse(open, &tokens[1..])?),
                    ))
                }
            }
        } else {
            if tokens.is_empty() {
                Ok(Object::Unspecified)
            } else if tokens[0] == "(" {
                if tokens.len() == 1 {
                    Err(DialError { code: 200 })
                } else {
                   self.parse(true, &tokens[1..])
                }
            } else if tokens[0] == ")" {
                Err(DialError { code: 201 })
            } else {
                self.parse_atom(&tokens[0])
            }
        }
    }    
}

#[cfg(test)]
mod tests {
    use super::*;


    fn testr(code: &str, expected: &str) -> Result<(), DialError> {
        let envr = create_global_envr();
        let reader = Reader { buffer: String::new()};
        let expr = reader.read(code)?;

        assert_eq!(format!("{}", expr.eval(envr.clone())?), expected);
        Ok(())
    }

    #[test]
    fn test_integer() -> Result<(), DialError> { testr("1", "1") }
    #[test]
    fn test_float() -> Result<(), DialError> { testr("3.1415", "3.1415") }

    #[test]
    fn test_string() -> Result<(), DialError> { testr("\"hello\"", "\"hello\"") }

    #[test]
    fn test_true() -> Result<(), DialError> { testr("#t", "#t") }

    #[test]
    fn test_false() -> Result<(), DialError> { testr("#f", "#f") }

    #[test]
    fn test_quote_number() -> Result<(), DialError> { testr("(quote 1)", "1") }

    #[test]
    fn test_built_in_add_as_proc() -> Result<(), DialError> { testr("+", "proc") }

    #[test]
    fn test_add() -> Result<(), DialError> { testr("(+ 1 2)", "3") }

}

fn main() {

    let envr = create_global_envr();
    let reader = Reader { buffer: String::new()};

    println!("dial");
    println!("(c) 2023, Omar Shorbaji");
    println!("version 0.1");

    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush();
        
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        let expr = reader.read(&line);

        match expr {
            Ok(expr) => {
                let value = expr.eval(envr.clone());

                match value {
                    Ok(r) => println!("{}", r),
                    Err(e) => println!("{}", e)
                };
            }
            Err(e) => println!("{}", e)
        };

    }
}
