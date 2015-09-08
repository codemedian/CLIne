use std::io::Read;
use std::collections::HashMap;

struct Cli<'a> {
    commands: HashMap<&'a str, Cli<'a>>
}

impl<'a> Cli<'a>{

    fn new() -> Cli<'a> {
        Cli {
            commands: HashMap::new()
        }
    }

    fn register(&mut self, command: Vec<&'a str>) {
        let mut it = command.iter();
        self._register(it);
    }

    fn _register(&mut self, mut it: std::slice::Iter<&'a str>) {
        if let Some(portion) = it.next() {
            if !self.commands.contains_key(portion) {
                self.commands.insert(portion, Cli { commands: HashMap::new() });
            }

            self.commands.get_mut(portion).unwrap()._register(it);
        }
    }

    fn suggest(&mut self, command: &str) -> Vec<&str> {
        let mut portions = command.trim().split(" ");
        self._suggest(&mut portions)
    }

    fn _suggest(&mut self, portions: &mut std::str::Split<&str>) -> Vec<&str> {
        let mut ret = Vec::with_capacity(self.commands.len());
       
        if let Some(portion) = portions.next() {
            if !portion.is_empty() {
                if let Some(cmd) = self.commands.get_mut(portion) {
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

fn main() {
    let mut cli = Cli::new();

    cli.register(vec!["show", "stuff"]);
    cli.register(vec!["show", "other"]);
    cli.register(vec!["list", "other", "cool"]);
    cli.register(vec!["list", "other", "uncool"]);

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        println!("got: {:?}", cli.suggest(&line));
    }
}
