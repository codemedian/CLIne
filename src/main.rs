use std::collections::HashMap;
use std::rc::Rc;
use std::str::SplitWhitespace;
use std::slice::Iter;
use std::cell::RefCell;

pub struct Command<'a> {
    command: Vec<&'a str>,
    exec: Box<RefCell<FnMut(Vec<&str>) + 'a>>,
    complete: Option<Box<FnMut(Vec<&str>) + 'a>>
}

impl<'a> Command<'a> {
    fn new<T: FnMut(Vec<&str>) + 'a, U: FnMut(Vec<&str>) + 'a>(cmd: Vec<&'a str>, exec_handler: T, complete_handler: Option<U>) -> Command<'a> {
        let mut obj = Command {
            command: cmd,
            exec: Box::new(RefCell::new(exec_handler)),
            complete: None
        };

        if let Some(cb) = complete_handler {
            obj.complete = Some(Box::new(cb));
        }

        obj
    }
}

pub struct Cli<'a> {
    commands: HashMap<&'a str, Cli<'a>>,
    handler: Option<Rc<Command<'a>>>
}

impl<'a> Cli<'a>{

    fn new() -> Cli<'a> {
        Cli {
            commands: HashMap::new(),
            handler: None
        }
    }

    fn register<T: FnMut(Vec<&str>) + 'a>(&mut self, cmd: Vec<&'a str>, exec: T) -> Result<(), ()> {
        let tmp = Rc::new(Command::new(cmd.clone(), exec, None::<T>));

        match self._register(cmd.iter(), tmp.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                //TODO: this can only happen if we register "Vec::new()" ... probably disallow that at all
                self.handler = Some(tmp.clone());
                Err(())
            }
        }
    }

    fn _register(&mut self, mut it: Iter<&'a str>, command: Rc<Command<'a>>) -> Result<(), ()> {
        if let Some(portion) = it.next() {
            let current:Vec<&str> = self.commands.keys().cloned().collect();

            if !self.commands.contains_key(portion) {
                let mut cli = Cli::new();
                if let Err(_) = cli._register(it, command.clone()) {
                    cli.handler = Some(command.clone())
                }
                self.commands.insert(portion, cli);
            } else {
                if let Some(cmd) = self.commands.get_mut(portion) {
                    if let Err(_) = cmd._register(it, command.clone()) {
                        cmd.handler = Some(command.clone())
                    }
                }
            };
            Ok(())
        } else {
            Err(())
        }
    }

    fn complete(&self, argv: &str) -> Vec<&str> {
        println!("complete for '{}'", argv.trim());
        let mut portions = argv.trim().split_whitespace();

        match self._complete(portions) {
            Ok(ret) => ret,
            Err(_) => Vec::new()
        }
    }

    fn _complete(&self, mut portions: SplitWhitespace) -> Result<Vec<&str>, ()> {
        if let Some(ref portion) = portions.next() {
            if let Some(ref cmd) = self.commands.get(portion) {
                cmd._complete(portions)
            } else {
                Ok(self.commands.keys()
                    .filter(|cmd| cmd.starts_with(portion))
                    .cloned()
                    .collect())
            }
        } else {
            Ok(self.commands.keys()
                .cloned()
                .collect())
        }
    }

    fn exec(&mut self, cmd: &str) {
        let portions = cmd.trim().split_whitespace();
        self._exec(portions);
    }

    fn _exec(&mut self, mut portions: SplitWhitespace) {
        if let Some(portion) = portions.next() {
            if let Some(cmd) = self.commands.get_mut(portion) {
                cmd._exec(portions);
            } else {
                if let Some(ref mut cb) = self.handler {
                    println!("handler for {:?}", cb.command);
                    (&mut *cb.exec.borrow_mut())(portions.collect());
                }
            }
        } else {
            if let Some(ref mut cb) = self.handler {
                println!("handler for {:?}", cb.command);
                (&mut *cb.exec.borrow_mut())(portions.collect());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_execute() {
        let mut called = false;
        {
            let mut cli = Cli::new();
            cli.register(vec!["foo bar"], | args | { called=true } );
            cli.exec("foo bar");
        }
        assert!(called == true)
    }
    
    #[test]
    fn test_complete_empty_str() {
        let mut cli = Cli::new();
        cli.register(vec!["foo"], | args | { } );
        cli.register(vec!["bar"], | args | { } );
        assert!(vec!["foo", "bar"] == cli.complete(""))
    }

    #[test]
    fn test_complete_partial() {
        let mut cli = Cli::new();
        cli.register(vec!["foo"], | args | { } );
        assert!(vec!["foo"] == cli.complete("f"));
        assert!(vec!["foo"] == cli.complete("fo"));
        assert!(vec!["foo"] == cli.complete("foo"));
    }
    
    #[test]
    fn test_complete_composite() {
        let mut cli = Cli::new();
        cli.register(vec!["foo", "bar"], | args | { } );
        assert!(vec!["foo", "bar"] == cli.complete("f"));
        assert!(vec!["foo", "bar"] == cli.complete("foo"));
        assert!(vec!["foo", "bar"] == cli.complete("foo "));
        assert!(vec!["foo", "bar"] == cli.complete("foo b"));
    }
}

fn foo(argv: Vec<&str>) {

}

fn main() {
    let mut cli = Cli::new();

    cli.register(vec!["show", "stuff"], foo).ok();
    cli.register(vec!["show", "other"], foo).ok();
    cli.register(vec!["some", "other"], foo).ok();
    cli.register(vec!["list", "other", "cool"], foo).ok();
    cli.register(vec!["list", "other", "uncool"], foo).ok();

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        //println!("got: {:?}", cli.complete(&line));
        cli.exec(&line);
    }
}
