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

// 
// Program and Exp
// 
// 

type _Program<'a>= Vec<Expr<'a>>;

#[derive(Clone)]
enum Expr<'a> {
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
    Pair(Box<Expr<'a>>, Box<Expr<'a>>)
    //    Proc(Proc),
}

impl<'a> std::fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Char(a) => write!(f, "{}", a),
            Expr::False => write!(f, "f"),
            Expr::Nil => write!(f, "nil"),
            Expr::Number(a) => write!(f, "{}", a),
            Expr::String(a) => write!(f, "{}", a),
            Expr::Symbol(a) => write!(f, "{}", a),
            Expr::True => write!(f, "t"),
            Expr::Pair(a, b) => write!(f, "({} . {})", a, b),
            _ => write!(f, "to be displayed"),
        }
    }
}

impl<'a> Expr<'a> {
    fn is_true(&self) -> bool {
        match self {
            Expr::Nil | Expr::False => false,
            _ => true,
        }
    }

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
    symbols: HashMap<String, Expr<'a>>,
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
    
    fn _bind(&mut self, key: String, value: Expr<'a>) {
        self.symbols.insert(key, value);
    }
    
    fn lookup(&self, key: &str) -> Option<&Expr> {
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

        let if_expr = Expr::Nil;
        let if_expr = Expr::cons(Expr::Number(Number::Exact(2, None)), if_expr);
        let if_expr = Expr::cons(Expr::Number(Number::Exact(1, None)), if_expr);
        let if_expr = Expr::cons(Expr::True, if_expr);
        let if_expr = Expr::cons(Expr::KeywordIf, if_expr);

        let quote_expr = Expr::Nil;
        let quote_expr = Expr::cons(Expr::cons(Expr::True, Expr::False), quote_expr);
        let quote_expr = Expr::cons(Expr::KeywordQuote, quote_expr);

        let _begin_expr = Expr::KeywordBegin;
        let _lambda_expr = Expr::KeywordLambda;
        let _set_expr = Expr::KeywordSet;
        let _symbol_expr = Expr::Symbol("x");

        let program = vec!(
            Expr::Nil,
            Expr::Char('a'),
            Expr::False,
            Expr::Nil,
            Expr::Number(Number::Exact(7, None)),
            Expr::Number(Number::Inexact(3.14)),
            Expr::String("hello"),
            Expr::True,
            if_expr,
            quote_expr,
            // begin_expr,
            // lambda_expr,
            // set_expr,
            // define_expr
            // symbol_expr,
            // Value::Symbol("+")),
        );

        Ok(program)
    }

    fn eval(&self, expr: &'a Expr, env: &'a Env) -> Result<&Expr, &'static str> {
        match expr {
            Expr::Char(_)   |
            Expr::False     |  
            Expr::Nil       |
            Expr::Number(_) |
            Expr::String(_) |
            Expr::True => Ok(&expr),
            Expr::Symbol(s) => match env.lookup(s) {
                Some(v) => Ok(v),
                None => Err("symbol not found"),
            },
            Expr::Pair(a, b) => {
                match **a {
                    Expr::KeywordIf => {
                        if self.eval(b.car().unwrap(), env).unwrap().is_true() {
                            self.eval(b.cadr().unwrap(), env)
                        } else {
                            self.eval(b.caddr().unwrap(), env)
                        }
                    },
                    Expr::KeywordQuote => {
                        Ok(b.car().unwrap())
                    },
                    _ => Err("don't know how to handle this pair")
                }
            },
            _ => Err("can't handle this"),
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
