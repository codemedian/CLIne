//! The `cline` crate provides an API that allows users to register CLI commands with an execute
//! and dynamic suggest callback to help implementing command line clients that support auto
//! completion of commands
//!
//! ## Getting started
//! The cline API works by creating an instance of the struct [`Cli`](struct.Cli.html) and calling
//! [`register`](struct.Cli.html#method.register), [`execute`](struct.Cli.html#method.execute) or
//! [`complete`](struct.Cli.html#method.complete) on it
//!
//! Create a new [`Cli`](struct.Cli.html) object:
//!
//! ```
//! # use cline::*;
//! let mut cli = Cli::new();
//! ```
//!
//! Register a function:
//!
//! ```
//! # use cline::*;
//! # let mut cli = Cli::new();
//! cli.register(vec!["list", "files"], | args | { println!("called with: {:?}", args); });
//! ```
//!
//! Get suggestions for autocompletion:
//!
//! ```
//! # use cline::*;
//! # let mut cli = Cli::new();
//! cli.complete("l"); //returns vec!["list"]
//! cli.complete("list"); //returns vec!["files"]
//! ```
//!
//! Execute command aka call registered exec closure:
//!
//! ```
//! # use cline::*;
//! # let mut cli = Cli::new();
//! cli.exec("list files"); //calls the registered closure
//! ```

use std::collections::HashMap;
use std::rc::Rc;
use std::str::SplitWhitespace;
use std::slice::Iter;
use std::cell::RefCell;

struct Command<'a> {
    command: Vec<&'a str>,
    exec: Box<RefCell<FnMut(Vec<&str>) + 'a>>,
    complete: Option<Box<RefCell<for <'b> FnMut(Vec<&'b str>) -> Vec<&'a str> + 'a>>>
}

impl<'a> Command<'a> {
    fn new<T: FnMut(Vec<&str>) + 'a, U: for <'b> FnMut(Vec<&'b str>) -> Vec<&'a str> + 'a>(cmd: Vec<&'a str>, exec_handler: T, complete_handler: Option<U>) -> Command<'a> {
        let mut complete_cb:Option<Box<RefCell<for <'b> FnMut(Vec<&'b str>) -> Vec<&'a str>>>> = None;
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

/// Opaque struct holding the registered Commands
pub struct Cli<'a> {
    commands: HashMap<&'a str, Cli<'a>>,
    handler: Option<Rc<Command<'a>>>
}

impl<'a> Cli<'a>{

    /// Constructs a new `Cli<'a>`
    ///
    /// This object holds all commands and callbacks registered with it
    pub fn new() -> Cli<'a> {
        Cli {
            commands: HashMap::new(),
            handler: None
        }
    }

    /// Registers a command with an exec callback
    /// When multiple commands get registered with matching prefixes, then a call to `complete`
    /// will return all "subcommands" available for the given input
    ///
    /// ```
    /// # use cline::*;
    /// # let mut cli = Cli::new();
    /// cli.register(vec!["foo", "bar"], | args | { println!("called with: {:?}", args); });
    /// // calling complete with "foo" would now return a Vec containing "bar"
    ///
    /// cli.register(vec!["foo", "baz"], | args | { println!("called with: {:?}", args); });
    /// // calling complete with "foo" would now return a Vec containing "bar" and "baz"
    /// ```
    ///
    pub fn register<T: FnMut(Vec<&str>) + 'a>(&mut self, cmd: Vec<&'a str>, exec: T) -> Result<(), ()> {
        let tmp = Rc::new(Command::new(cmd.clone(), exec, None::<for <'b> fn(Vec<&'b str>) -> Vec<&'a str>>));

        match self._register(cmd.iter(), tmp.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                //TODO: this can only happen if we register "Vec::new()" ... probably disallow that at all
                self.handler = Some(tmp.clone());
                Err(())
            }
        }
    }

    /// Registers a command with an exec callback and a dynamic complete callback
    /// This comes in handy if the completion is based on data that can change at runtime like a
    /// list of active sessions
    ///
    /// ```
    /// # use cline::*;
    /// # let mut cli = Cli::new();
    /// cli.register_dyn_complete(vec!["foo"],
    ///     | args | { println!("called with: {:?}", args); },
    ///     | args | { vec!["bar", "baz"] });
    ///
    /// cli.complete("foo"); // --> vec!["bar", "baz"]
    /// ```
    pub fn register_dyn_complete<T: FnMut(Vec<&str>) + 'a, U: for <'b> FnMut(Vec<&'b str>) -> Vec<&'a str> + 'a>(&mut self, cmd: Vec<&'a str>, exec: T, complete: U) -> Result<(), ()> {
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

    /// Returns a Vector with "subcommands" that can either complete the current work or follow the given input
    /// for registered commands
    ///
    /// ```
    /// # use cline::*;
    /// # let mut cli = Cli::new();
    /// cli.register(vec!["foo","bar"], | _ | {});
    /// assert!(vec!["foo"] == cli.complete("f"));
    /// assert!(vec!["bar"] == cli.complete("foo"));
    ///
    /// cli.register(vec!["foo","baz"], | _ | {}); // -> vec!["bar", "baz"]
    /// ```
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
                    args.push(portion);
                    args.extend(portions);
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

    /// Calls the execute callback registered with a command specified by `cmd`
    ///
    /// ```
    /// # use cline::*;
    /// # let mut cli = Cli::new();
    /// cli.register(vec!["foo"], | args | { println!("called with: {:?}", args); });
    /// cli.exec("foo");
    /// ```
    pub fn exec<'b>(&mut self, cmd: &'b str) {
        let argv:Vec<&str> = cmd.clone().split_whitespace().collect();
        let portions = cmd.trim().split_whitespace();
        self._exec(portions, argv);
    }


    //TODO: don't wanna pass through argv - do it like complete
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

/// Helper function that emulates linux terminal behaviour for command 
/// completion based on the commands registered with the [`Cli`](struct.Cli.html) 
/// struct passed to the function.
/// Can be exited with Ctrl + c
///
/// ```ignore
/// # use cline::{cli, cline_run};
/// # let mut cli = Cli::new();
/// cli.register(vec!["foo", "bar"], | _ | { println!("running foo bar") });
/// cli.register(vec!["foo", "baz"], | _ | { println!("running foo baz") });
///
/// cline_run(&mut cli);
/// ```
/// # Note
/// Current implementation only works on linux (`termios` based)
#[cfg(unix)]
pub fn cline_run(cli: &mut Cli) {
    unix::unix_cline_run(cli);
}
#[cfg(windows)]
pub fn cline_run(cli: &mut Cli) {
    panic!("Not yet implemented");
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug)]
pub enum Key {
    Char(char),
    Symbol(char),
    Digit(u8),
    Arrow(Direction),
    Whitespace,
    Backspace,
    Del,
    Tab,
    Newline,
    Etx
}

#[cfg(unix)]
mod unix {
    extern crate termios;

    use std::io::{stdout, stdin, Bytes};
    use std::io::prelude::*;
    use self::termios::*;
    use super::*;

    fn read_key<T: Read>(bytes: &mut Bytes<T>) -> Option<Key> {
        match bytes.next() {
            Some(Ok(0x1B)) => { //Esc
                match bytes.next() {
                    Some(Ok(0x5b)) => { // [
                        match bytes.next() {
                            Some(Ok(0x41)) => { // A
                                Some(Key::Arrow(Direction::Up))
                            },
                            Some(Ok(0x42)) => { // B
                                Some(Key::Arrow(Direction::Down))
                            },
                            Some(Ok(0x43)) => { // C
                                Some(Key::Arrow(Direction::Right))
                            },
                            Some(Ok(0x44)) => { // D
                                Some(Key::Arrow(Direction::Left))
                            },
                            c @ _ => {
                                println!("{} {} {}", 0x1B, 0x5b, c.unwrap().unwrap());
                                None
                            }
                        }
                    },
                    c @ _ => {
                        println!("{} {}", 0x1B, c.unwrap().unwrap());
                        None
                    }
                }
            },
            Some(Ok(c)) if c >= 0x30 && c <= 0x39 => {
                Some(Key::Digit(c - 0x30))
            },
            Some(Ok(c)) if (c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A) => {
                Some(Key::Char(c as char))
            },
            Some(Ok(0x20)) => {
                Some(Key::Whitespace)
            },
            Some(Ok(0x7f)) => {
                Some(Key::Del)
            },
            Some(Ok(0x09)) => {
                Some(Key::Tab)
            },
            Some(Ok(0x08)) => {
                Some(Key::Backspace)
            },
            Some(Ok(0xA)) => {
                Some(Key::Newline)
            },
            Some(Ok(0x3)) => {
                Some(Key::Etx)
            },
            Some(Ok(c)) => {
                println!("{:?}", c);
                Some(Key::Symbol(c as char))
            },
            _ => {
                None
            }
        }
    }

    pub fn unix_cline_run(cli: &mut Cli) {
        let mut termios = Termios::from_fd(0).unwrap();
        let term_orig = termios;
        let mut input_iter = stdin().bytes();
        let mut command = String::new();

        termios.c_lflag &= !(ICANON | IEXTEN | ISIG | ECHO);
        tcsetattr(0, TCSANOW, &termios).unwrap();
        tcflush(0, TCIOFLUSH).unwrap();

        print!(">> ");
        stdout().flush().unwrap();

        loop {
            match read_key(&mut input_iter) {
                Some(key) => {
                    match key {
                        Key::Char(c) => {
                            command.push(c);
                            print!("{}", c);
                            stdout().flush().unwrap();
                        },
                        Key::Whitespace => {
                            command.push(' ');
                            print!(" ");
                            stdout().flush().unwrap();
                        }
                        Key::Del | Key::Backspace => {
                            command.pop();
                            print!("{} {}", 0x08 as char, 0x08 as char);
                            stdout().flush().unwrap();
                        },
                        Key::Tab => {
                            let suggestions = cli.complete(&command);
                            print!("{}", '\n');
                            for cmd in suggestions.iter() {
                                print!("{} ", cmd);
                            }
                            print!("\n>> {}", command);
                            stdout().flush().unwrap();
                        },
                        Key::Newline => {
                            println!("");
                            cli.exec(&command);
                            command.clear();
                            print!(">> ");
                            stdout().flush().unwrap();
                        },
                        Key::Etx => { //Ctrl + C
                            println!("");
                            break;
                        }
                        x @ _ => {
                            println!("unhandled: {:?}", x);
                        }
                    }
                },
                None => {
                    break;
                }
            }
        }
        tcsetattr(0, termios::TCSANOW, &term_orig).unwrap();
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
        cli.register_dyn_complete(vec!["foo"], | _ | { }, | _ | {
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
