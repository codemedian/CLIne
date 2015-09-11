use std::io::Read;
use std::collections::HashMap;
use std::str::Split;

pub struct Command<'a> {
    command: &'a str,
    exec: Box<Fn(Vec<&str>)>,
    complete: Option<Box<Fn(Vec<&str>)>>
}

impl<'a> Command<'a> {
    fn new<T: Fn(Vec<&str>) + 'static, U: Fn(Vec<&str>) + 'static>(cmd: &'a str, exec_handler: T, complete_handler: Option<U>) -> Command {
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
    commands: HashMap<&'a str, Command<'a>>
}

impl<'a> Cli<'a>{

    fn new() -> Cli<'a> {
        Cli {
            commands: HashMap::new()
        }
    }

    fn register<T: Fn(Vec<&str>) + 'static>(&mut self, cmd: &'a str, exec: T) -> Result<(), Command> {

        let tmp = Command { 
            command: cmd,
            exec: Box::new(exec),
            complete: None
        };

        match self.commands.insert(cmd, tmp) {
            Some(existing) => Err(existing),
            None => Ok(())
        }
    }

    fn complete(&self, argv: &str) -> Vec<&str> {
        Vec::new()
    }

    fn exec(&self, cmd: &str) {
        if let Some(command) = self.commands.get(cmd) {
            //TODO: figure out why I can't do that in one line
            let ref x = command.exec;
            x(vec!["blah"]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testRegister() {
        let mut cli = Cli::new();
        let mut called = false;

        cli.register("my first command", | args | { println!("called"); } );
        cli.exec("my first command");

        assert!(called == true)
    }
}

    //fn new(callback: Option<Box<Fn()>>) -> Cli<'a> {
        //Cli {
            //commands: HashMap::new(),
            //exec: callback,
            //complete_cb: None,
        //}
    //}

    //fn register<T: Fn() + 'static>(&mut self, command: Vec<&'a str>, callback: T) {
        //let mut it = command.iter();
        //self._register(it, callback);
    //}

    //fn _register<T: Fn() + 'static>(&mut self, mut it: std::slice::Iter<&'a str>, callback: T) {
        //if let Some(portion) = it.next() {
            //if !self.commands.contains_key(portion) {
                //self.commands.insert(portion, Cli::new(Some(Box::new(callback))));
            //}

            //self.commands.get_mut(portion).unwrap()._register(it, callback);
        //}
    //}

    //fn suggest(&mut self, command: &str) -> Vec<&str> {
        //let mut portions = command.trim().split(" ");
        //let mut suggestions = self._suggest(&mut portions);
        //if let Some(ref cb) = self.complete_cb {
            //cb();
            //println!("got callback");
        //}
        //suggestions
    //}

    //fn _suggest(&self, portions: &mut std::str::Split<&str>) -> Vec<&str> {
        //let mut ret = Vec::with_capacity(self.commands.len());
       
        //if let Some(portion) = portions.next() {
            //if !portion.is_empty() {
                //if let Some(cmd) = self.commands.get(portion) {
                    //ret = cmd._suggest(portions);
                //}
            //} else {
                //for key in self.commands.keys() {
                    //ret.push(*key);
                //}
            //}
        //} else {
            //for key in self.commands.keys() {
                //ret.push(*key);
            //}
        //}

        //ret
    //}
//}

//fn foo() {

//}

fn main() {
    //let mut cli = Cli::new(Some(Box::new(foo)));

    //cli.register(vec!["show", "stuff"], foo);
    //cli.register(vec!["show", "other"], foo);
    //cli.register(vec!["list", "other", "cool"], foo);
    //cli.register(vec!["list", "other", "uncool"], foo);

    //loop {
        //let mut line = String::new();
        //std::io::stdin().read_line(&mut line).unwrap();

        //println!("got: {:?}", cli.suggest(&line));
    //}
}
