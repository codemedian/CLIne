extern crate cline;
extern crate termios;

use std::io;
use std::io::prelude::*;
use std::os::unix::io::RawFd;
use cline::Cli;
use termios::*;

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug)]
enum Key {
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

fn read_key<T: Read>(bytes: &mut std::io::Bytes<T>) -> Option<Key> {
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


fn run(cli: &mut Cli) {
    let mut termios = Termios::from_fd(0).unwrap();
    let term_orig = termios;
    let mut input_iter = io::stdin().bytes();
    let mut command = String::new();

    termios.c_lflag &= !(ICANON | IEXTEN | ISIG | ECHO);
    tcsetattr(0, TCSANOW, &termios);
    tcflush(0, TCIOFLUSH);

    print!(">> ");
    io::stdout().flush();

    loop {
        match(read_key(&mut input_iter)) {
            Some(key) => {
                match key {
                    Key::Char(c) => {
                        command.push(c);
                        print!("{}", c);
                        io::stdout().flush();
                    },
                    Key::Whitespace => {
                        command.push(' ');
                        print!(" ");
                        io::stdout().flush();
                    }
                    Key::Del | Key::Backspace => {
                        command.pop();
                        print!("{} {}", 0x08 as char, 0x08 as char);
                        io::stdout().flush();
                    },
                    Key::Tab => {
                        let suggestions = cli.complete(&command);
                        print!("{}", '\n');
                        for cmd in suggestions.iter() {
                            print!("{} ", cmd);
                        }
                        print!("\n>> {}", command);
                        io::stdout().flush();
                    },
                    Key::Newline => {
                        println!("");
                        cli.exec(&command);
                        command.clear();
                        print!(">> ");
                        io::stdout().flush();
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
    tcsetattr(0, termios::TCSANOW, &term_orig);
}

fn main() {
    let mut cli = Cli::new();

    cli.register(vec!["foo", "bar"], | _ | { println!("running foo bar") });
    cli.register(vec!["foo", "baz"], | _ | { println!("running foo baz") });

    run(&mut cli);
}
