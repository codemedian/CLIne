use std::io::Read;
use std::collections::HashMap;
use std::str::Split;

struct Cli<'a> {
    commands: HashMap<&'a str, Cli<'a>>,
    exec: Option<Box<Fn()>>,
    complete_cb: Option<Box<Fn()>>
}

impl<'a> Cli<'a>{

    fn new(callback: Option<Box<Fn()>>) -> Cli<'a> {
        Cli {
            commands: HashMap::new(),
            exec: callback,
            complete_cb: None,
        }
    }

    fn register(&mut self, command: Vec<&'a str>, callback: Fn()) {
        let mut it = command.iter();
        self._register(it, callback);
    }

    fn _register(&mut self, mut it: std::slice::Iter<&'a str>, callback: Fn()) {
        if let Some(portion) = it.next() {
            if !self.commands.contains_key(portion) {
                self.commands.insert(portion, Cli::new(Some(Box::new(callback))));
            }

            self.commands.get_mut(portion).unwrap()._register(it, callback);
        }
    }

    fn suggest(&mut self, command: &str) -> Vec<&str> {
        let mut portions = command.trim().split(" ");
        let mut suggestions = self._suggest(&mut portions);
        if let Some(ref cb) = self.complete_cb {
            cb();
            println!("got callback");
        }
        suggestions
    }

    fn _suggest(&self, portions: &mut std::str::Split<&str>) -> Vec<&str> {
        let mut ret = Vec::with_capacity(self.commands.len());
       
        if let Some(portion) = portions.next() {
            if !portion.is_empty() {
                if let Some(cmd) = self.commands.get(portion) {
                    ret = cmd._suggest(portions);
                }
            } else {
                for key in self.commands.keys() {
                    ret.push(*key);
                }
            }
        } else {
            for key in self.commands.keys() {
                ret.push(*key);
            }
        }

        ret
    }
}

fn foo(args: Vec<&str>) {

}

fn main() {
    let mut cli = Cli::new(Box::new(foo));

    cli.register(vec!["show", "stuff"], foo);
    cli.register(vec!["show", "other"], foo);
    cli.register(vec!["list", "other", "cool"], foo);
    cli.register(vec!["list", "other", "uncool"], foo);

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        println!("got: {:?}", cli.suggest(&line));
    }
}
