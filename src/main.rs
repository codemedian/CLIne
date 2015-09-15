use std::collections::HashMap;
use std::str::SplitWhitespace;
use std::slice::Iter;

pub struct Command<'a> {
    command: Vec<&'a str>,
    exec: Box<FnMut(Vec<&str>) + 'a>,
    complete: Option<Box<FnMut(Vec<&str>) + 'a>>
}

impl<'a> Command<'a> {
    fn new<T: FnMut(Vec<&str>) + 'a, U: FnMut(Vec<&str>) + 'a>(cmd: Vec<&'a str>, exec_handler: T, complete_handler: Option<U>) -> Command<'a> {
        let mut obj = Command {
            command: cmd,
            exec: Box::new(exec_handler),
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
    handler: Option<Command<'a>>
}

impl<'a> Cli<'a>{

    fn new() -> Cli<'a> {
        Cli {
            commands: HashMap::new(),
            handler: None
        }
    }

    fn register<T: FnMut(Vec<&str>) + 'a>(&mut self, cmd: Vec<&'a str>, exec: T) -> Result<(), ()> {
        //let tmp = Command::new(cmd, exec, None);
        let tmp = Command { 
            command: cmd.clone(),
            exec: Box::new(exec),
            complete: None
        };


        match self._register(cmd.iter(), &tmp) {
            Ok(_) => Ok(()),
            Err(_) => {
                //this only happens if we register "Vec::new()" ... probably disallow that at all
                self.handler = Some(tmp);
                Err(())
            }
        }
    }

    fn _register(&mut self, mut it: Iter<&'a str>, command: &Command) -> Result<(), ()> {
        if let Some(portion) = it.next() {
            let current:Vec<&str> = self.commands.keys().cloned().collect();

            if !self.commands.contains_key(portion) {
                let mut cli = Cli::new();
                cli._register(it, command);
                self.commands.insert(portion, cli);
            } else {
                if let Some(cmd) = self.commands.get_mut(portion) {
                    //TODO: register handler if we get an error
                    cmd._register(it, command);
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
        let mut portions = cmd.trim().split_whitespace();
        self._exec(portions);
        //if let Some(command) = self.commands.get_mut(cmd) {
            ////TODO: figure out why I can't do that in one line
            ////let ref mut x = command.exec;
            ////x(vec!["blah"]);
            //println!("found cmd");
        //} else {
            //println!(" not found cmd");
        //}
    }

    fn _exec(&mut self, mut portions: SplitWhitespace) {
        if let Some(portion) = portions.next() {
            if let Some(cmd) = self.commands.get_mut(portion) {
                cmd._exec(portions);
            } else {
                match self.handler {
                    Some(ref x) => println!("handler for {:?}", x.command),
                    None => println!("no handler")
                }
            }
        } else {
            match self.handler {
                Some(ref x) => println!("handler for {:?}", x.command),
                None => println!("no handler")
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

    cli.register(vec!["show", "stuff"], foo);
    cli.register(vec!["show", "other"], foo);
    cli.register(vec!["some", "other"], foo);
    cli.register(vec!["list", "other", "cool"], foo);
    cli.register(vec!["list", "other", "uncool"], foo);

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        println!("got: {:?}", cli.complete(&line));
    }
}
