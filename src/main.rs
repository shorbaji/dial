use std::collections::HashMap;

// 
// Value
// 
// 
#[derive(Copy, Clone)]
enum Number {
    Exact(i128, Option<i128>),
    Inexact(f64),
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Exact(n, d) => match d {
                Some(d) => write!(f, "{}/{}", n, d),
                None => write!(f, "{}", n),
            },
            Number::Inexact(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Copy, Clone)]
enum Value<'a> {
    Char(char),
    False,
    KeywordBegin,
    KeywordIf,
    KeywordLambda,
    KeywordQuote,
    KeywordSet,
    Nil,
    Number(Number),
    String(&'static str),
    Symbol(&'a str),
    True,
    //    Proc(Proc),
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Char(a) => write!(f, "{}", a),
            Value::False => write!(f, "f"),
            Value::Nil => write!(f, "nil"),
            Value::Number(a) => write!(f, "{}", a),
            Value::String(a) => write!(f, "{}", a),
            Value::Symbol(a) => write!(f, "{}", a),
            Value::True => write!(f, "f"),
            _ => write!(f, "to be displayed"),
        }
    }
}

impl<'a> Value<'a> {
    fn is_true(self) -> bool {
        match self {
            Value::Nil | Value::False => false,
            _ => true,
        }
    }

}

// 
// Program and Exp
// 
// 

type _Program<'a>= Vec<Expr<'a>>;

enum Expr<'a> {
    Atom(Value<'a>),
    Pair(Box<Expr<'a>>, Box<Expr<'a>>)
}

impl<'a> Expr<'a> {

    fn cons(a: Expr<'a>, b: Expr<'a>) -> Self {
        Expr::Pair(Box::new(a), Box::new(b))
    }

    fn car(&self) -> Result<&Expr, &'static str> {
        match self {
            Expr::Pair(a, _) => Ok(&*a),
            _ => Err("can't car an atom")
        }
    }

    fn cdr(&self) -> Result<&Expr, &'static str> {
        match self {
            Expr::Pair(_, b) => Ok(&*b),
            _ => Err("can't car an atom")
        }
    }

    fn cadr(&self) -> Result<&Expr, &'static str> {
        self.cdr().unwrap().car()
    }

    fn caddr(&self) -> Result<&Expr, &'static str> {
        self.cdr().unwrap().cdr().unwrap().car()
    }

}

// 
// Env
// 
// 

struct Env<'a> {
    symbols: HashMap<String, Value<'a>>,
    _parent: Option<Box<Env<'a>>>,
}

impl<'a> Default for Env<'a> {
    fn default() -> Self {
        Self {
            symbols: HashMap::new(),
            _parent: None,
        }
    }
}

impl<'a> Env<'a> {
    fn new() -> Self {
        Default::default()
    }
    
    fn _bind(&mut self, key: String, value: Value<'a>) {
        self.symbols.insert(key, value);
    }
    
    fn lookup(&self, key: &str) -> Option<&Value> {
        self.symbols.get(key)
    }
}

// 
// User
// 
// 
type User = String;

// 
// Machine
// 
// 
struct Machine<'a> {
    _name: String,
    global_env: Env<'a>,
}

impl<'a> Machine<'a> {
    fn compute(&mut self, code: &str) {

        let program = self.read(code).expect("read error");

        for exp in program {
            println!("{}", self.eval(&exp, &self.global_env).expect("cannot compute"));
        }
    }
    
    fn read(&self, _code: &str) -> Result<Vec<Expr>, &str> {

        let if_expr = Expr::Atom(Value::Nil);
        let if_expr = Expr::cons(Expr::Atom(Value::Number(Number::Exact(2, None))), if_expr);
        let if_expr = Expr::cons(Expr::Atom(Value::Number(Number::Exact(1, None))), if_expr);
        let if_expr = Expr::cons(Expr::Atom(Value::True), if_expr);
        let if_expr = Expr::cons(Expr::Atom(Value::KeywordIf), if_expr);

        let _begin_expr = Expr::Atom(Value::KeywordBegin);
        let _lambda_expr = Expr::Atom(Value::KeywordLambda);
        let _quote_expr = Expr::Atom(Value::KeywordQuote);
        let _set_expr = Expr::Atom(Value::KeywordSet);
        let _symbol_expr = Expr::Atom(Value::Symbol("x"));

        let program = vec!(
            Expr::Atom(Value::Nil),
            Expr::Atom(Value::Char('a')),
            Expr::Atom(Value::False),
            Expr::Atom(Value::Nil),
            Expr::Atom(Value::Number(Number::Exact(7, None))),
            Expr::Atom(Value::Number(Number::Inexact(3.14))),
            Expr::Atom(Value::String("hello")),
            Expr::Atom(Value::True),
            if_expr,
            // begin_expr,
            // lambda_expr,
            // quote_expr,
            // set_expr,
            // define_expr
            // symbol_expr,
            // Expr::Atom(Value::Symbol("+")),

        );

        Ok(program)
    }

    fn eval(&self, expr: &'a Expr, env: &'a Env) -> Result<Value, &'static str> {
        match expr {
            Expr::Atom(atom) =>  {
                match atom {
                    Value::Char(_)   |
                    Value::False     |  
                    Value::Nil       |
                    Value::Number(_) |
                    Value::String(_) |
                    Value::True => Ok(*atom),
                    Value::Symbol(s) => match env.lookup(s) {
                        Some(v) => Ok(*v),
                        None => Err("symbol not found"),
                    }
                    _ => Err("can't handle this atom"),
                }
            },
            Expr::Pair(a, b) => {
                match **a {
                    Expr::Atom(Value::KeywordIf) => {
                        if self.eval(b.car().unwrap(), env).unwrap().is_true() {
                            self.eval(b.cadr().unwrap(), env)
                        } else {
                            self.eval(b.caddr().unwrap(), env)
                        }
                    },
                    _ => Err("don't know how to handle this pair")
                }
            },
        }
    }
}

// 
// Cloud
// 
// 
struct Cloud<'a> {
    machines: HashMap<String, Machine<'a>>,
    _users: HashMap<i128, User>,
}

impl<'a> Default for Cloud<'a> {
    fn default() -> Self {
        Self {
            machines: HashMap::new(),
            _users: HashMap::new(),
        }
    }
}

impl<'a> Cloud<'a> {
    fn new() -> Self {
        Default::default()
    }
    
    fn create_machine(&mut self, name: &str){
        let machine = Machine {
            _name: name.to_string(),
            global_env: Env::new(),
        };
        
        self.machines.insert(name.to_string(), machine);
    }
    
    fn get_mut_machine(&mut self, name: &str) -> Option<&mut Machine<'a>> {
        self.machines.get_mut(&name.to_string())
    }
    
    fn compute(&mut self, name: &str, program: &str) {
        self.get_mut_machine(name).expect("can't find machine").compute(program);
    }
}

fn main() {
    // big bang
    let mut cloud: Cloud = Cloud::new();
    
    // first machines 
    cloud.create_machine("/core");
    cloud.create_machine("/users/shorbaji");
    
    // first eval
    
    let code = "1";

    cloud.compute("/core", code);    
}
