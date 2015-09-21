use std::collections::HashMap;
use std::rc::Rc;
use std::str::SplitWhitespace;
use std::slice::Iter;
use std::cell::RefCell;

pub struct Command<'a> {
    command: Vec<&'a str>,
    exec: Box<RefCell<FnMut(Vec<&str>) + 'a>>,
    complete: Option<Box<RefCell<FnMut(Vec<&str>) -> Vec<&str> + 'a>>>
}

impl<'a> Command<'a> {
    fn new<T: FnMut(Vec<&str>) + 'a, U: FnMut(Vec<&str>) -> Vec<&str> + 'a>(cmd: Vec<&'a str>, exec_handler: T, complete_handler: Option<U>) -> Command<'a> {
        let mut complete_cb:Option<Box<RefCell<FnMut(Vec<&str>) -> Vec<&str>>>> = None;
        if let Some(cb) = complete_handler {
            complete_cb = Some(Box::new(RefCell::new(cb)));
        }

        Command {
            command: cmd,
            exec: Box::new(RefCell::new(exec_handler)),
            complete: complete_cb
        }
    }
}

pub struct Cli<'a> {
    commands: HashMap<&'a str, Cli<'a>>,
    handler: Option<Rc<Command<'a>>>
}

impl<'a> Cli<'a>{

    pub fn new() -> Cli<'a> {
        Cli {
            commands: HashMap::new(),
            handler: None
        }
    }

    pub fn register<T: FnMut(Vec<&str>) + 'a>(&mut self, cmd: Vec<&'a str>, exec: T) -> Result<(), ()> {
        let tmp = Rc::new(Command::new(cmd.clone(), exec, None::<fn(Vec<&str>) -> Vec<&str>>));

        match self._register(cmd.iter(), tmp.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                //TODO: this can only happen if we register "Vec::new()" ... probably disallow that at all
                self.handler = Some(tmp.clone());
                Err(())
            }
        }
    }

    fn register_dyn_complete<T: FnMut(Vec<&str>) + 'a, U: FnMut(Vec<&str>) -> Vec<&str> + 'a>(&mut self, cmd: Vec<&'a str>, exec: T, complete: U) -> Result<(), ()> {
        let tmp = Rc::new(Command::new(cmd.clone(), exec, Some(complete)));

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

    pub fn complete<'b>(&mut self, argv: &'b str) -> Vec<&'a str> {
        let portions = argv.trim().split_whitespace();

        match self._complete(portions) {
            Ok(ret) => ret,
            Err(_) => Vec::new()
        }
    }

    fn _complete<'b>(&mut self, mut portions: SplitWhitespace<'b>) -> Result<Vec<&'a str>, ()> {
        if let Some(ref portion) = portions.next() {
            if let Some(cmd) = self.commands.get_mut(*portion) {
                return cmd._complete(portions);
            }
            
            let mut ret:Vec<&str> = Vec::new();
            if let Some(ref mut handler) = self.handler {
                if let Some(ref cb) = handler.complete {
                    let mut args:Vec<&str> = Vec::new();
                    //args.push(portion);
                    //args.extend(portions);
                    ret.extend((&mut *cb.borrow_mut())(args.clone()));
                }
            }
            ret.extend(self.commands.keys()
                .filter(|cmd| cmd.starts_with(portion)));
            return Ok(ret);
        } else {
            let mut ret:Vec<&str> = Vec::new();
            if let Some(ref mut handler) = self.handler {
                if let Some(ref cb) = handler.complete {
                    ret.extend((&mut *cb.borrow_mut())(vec![""]).iter());
                }
            }
            ret.extend(self.commands.keys());
            return Ok(ret)
        }
    }

    pub fn exec<'b>(&mut self, cmd: &'b str) {
        let argv:Vec<&str> = cmd.clone().split_whitespace().collect();
        let portions = cmd.trim().split_whitespace();
        self._exec(portions, argv);
    }

    fn _exec<'b>(&mut self, mut portions: SplitWhitespace<'b>, argv: Vec<&str>) {
        if let Some(ref portion) = portions.next() {
            if let Some(cmd) = self.commands.get_mut(*portion) {
                cmd._exec(portions, argv);
            } else {
                if let Some(ref mut cb) = self.handler {
                    (&mut *cb.exec.borrow_mut())(argv);
                }
            }
        } else {
            if let Some(ref mut cb) = self.handler {
                (&mut *cb.exec.borrow_mut())(argv);
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
            cli.register(vec!["foo", "bar"], | _ | { called=true }).ok();
            cli.exec("foo bar");
        }
        assert!(called == true)
    }

    #[test]
    fn test_register_and_execute_multiple_times() {
        let mut called = 0u8;
        {
            let mut cli = Cli::new();
            cli.register(vec!["foo", "bar"], | _ | { called = called + 1} ).ok();
            cli.exec("foo bar");
            cli.exec("foo bar");
        }
        assert!(called == 2)
    }

    #[test]
    fn test_complete_empty_single_cmd() {
        let mut cli = Cli::new();
        cli.register(vec!["foo"], | _ | { } ).ok();
        assert!(vec!["foo"] == cli.complete(""));
        assert!(vec!["foo"] == cli.complete("f"));
        assert!(vec!["foo"] == cli.complete("  f"));
        //assert!(vec![""] == cli.complete("f   "));  //FIXME: this test should pass
        assert!(cli.complete("foo").is_empty());
    }

    #[test]
    fn test_complete_with_dynamic() {
        let mut cli = Cli::new();
        cli.register_dyn_complete(vec!["foo"], | _ | { }, | args | {
            vec!["bar", "baz"]
        }).ok();
        assert!(vec!["foo"] == cli.complete("f"));
        assert!(vec!["bar", "baz"] == cli.complete("foo a b"));
    }

    #[test]
    fn test_complete_multi_cmd() {
        let mut cli = Cli::new();
        cli.register(vec!["foo", "bar"], | _| { } ).ok();
        assert!(vec!["foo"] == cli.complete("f"));
        assert!(vec!["bar"] == cli.complete("foo"));
        assert!(vec!["bar"] == cli.complete("foo b"));
    }

    #[test]
    fn test_register_and_execute_with_arguments() {
        let mut called = false;
        {
            let mut cli = Cli::new();
            cli.register(vec!["foo"], | args | {
                called=true;
                assert!(vec!["foo", "bar", "baz"] == args);
            } ).ok();
            cli.exec("foo bar baz");
        }
        assert!(called == true);
    }
}

