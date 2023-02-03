use std::collections::HashMap;

struct Cloud {
    machines: HashMap<String, Machine>,
    users: HashMap<i128, User>,
}

impl Default for Cloud {
    fn default() -> Self {
        Self {
            machines: HashMap::new(),
            users: HashMap::new(),
        }
    }
}

impl Cloud {
    fn new() -> Self {
        Default::default()
    }

    fn create_machine(&mut self, name: &str){
        let machine = Machine {
            name: name.to_string(),
            global_env: Env::new(),
        };

        self.machines.insert(name.to_string(), machine);
    }

    fn get_mut_machine(&mut self, name: &str) -> Option<&mut Machine> {
        self.machines.get_mut(&name.to_string())
    }

    fn compute(&mut self, name: &str, program: Program) {
        self.get_mut_machine(name).expect("can't find machine").compute(program);
    }
}

type User = String;
struct Machine {
    name: String,
    global_env: Env,
}

impl Machine {
    fn compute(&mut self, program: Program) {
        for exp in program {
            self.eval(exp, &self.global_env);
        }
    }

    fn eval(&self, exp: Exp, env: &Env) -> Option<Value> {
        match exp {
            Exp::Number(n) => Some(Value::Number(n)),
            Exp::Symbol(s) => Some(env.lookup(s).expect("not found").clone()),
            // Exp::Pair(pair) => 
            //     match pair.car {
            //         Exp::KeywordBegin => Err("don't know how to begin"),
            //         Exp::KeywordIf => Err("don't know how to if"),
            //         Exp::KeywordLambda => Err("don't know how to Lambda"),
            //         Exp::KeywordQuote => Err("don't know how to quote"),
            //         Exp::KeywordSet => Err("don't know how to set!"),
            //         _ => Err("don't know how to function application"),
            //     }
            _ => None,
    }
}
}
struct Env {
    map: Vec<(String, Value)>,
    parent: Option<Box<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            map: Vec::new(),
            parent: None,
        }
    }
}

impl Env {
    fn new() -> Self {
        Default::default()
    }

    fn bind(&mut self, symbol: String, value: Value) {
        self.map.push((symbol, value));
    }

    fn lookup(&self, symbol: String) -> Option<&Value> {
        self.map
            .iter()
            .find(|(k, _)| k.eq(&symbol))
            .map(|(_, v)| v)
    }
}

type Program = Vec<Exp>;

enum Exp {
    KeywordBegin,
    KeywordIf,
    KeywordLambda,
    KeywordQuote,
    KeywordSet,
    Number(Number),
    Symbol(String),
}

#[derive(Clone)]
enum Number {
    Exact(i128, Option<i128>),
    Inexact(f64),
}

#[derive(Clone)]
enum Value{
    Bool(bool),
    Char(char),
    Nil,
    Number(Number),
    String(String),
    //    Proc(Proc),
//    String(Box<String>),
}

// fn print(exp: Box<Exp>) -> String{
//     match *exp {
//         Exp::Number(n) => fmt!("{}", n),
//         _ => String::from("don't know how to print")


//     }
// }


// struct Pair<'a> {
//     car: &'a Exp,
//     cdr: &'a Exp,
// }

// impl <'a> Pair<'a> {
//     fn car(&self) -> Result<&Exp, &'static str> {
//         Ok(self.car)
//     }
// }


fn main() {
    // big bang
    let mut cloud: Cloud = Cloud::new();

    // first machines 
    cloud.create_machine("core");
    cloud.create_machine("shorbaji");

    // first eval
    let program = vec!(Exp::Number(Number::Exact(42, None)));

    cloud.compute("core", program);

}
