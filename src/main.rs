extern crate cline;
extern crate termios;

use std::io;
use std::io::prelude::*;
use std::os::unix::io::RawFd;
use termios::*;

fn main() {
    let mut termios = Termios::from_fd(0).unwrap();
    let term_orig = termios;
    let mut cli = cline::Cli::new();
    let mut buf:Vec<u8> = Vec::new();

    cli.register(vec!["show", "stuff"], | _ | { println!("wooo haaa") });
    cli.register(vec!["show", "other"], | _ | { println!("wooo haaa") });
    cli.register(vec!["foo", "bar"], | _ | { println!("wooo haaa") });
    cli.register(vec!["exit"], | _ | { 
        tcsetattr(0, termios::TCSANOW, &term_orig);
        std::process::exit(0); 
    });
   


    //termios.c_lflag = ECHONL;
    termios.c_lflag &= !(ICANON | IEXTEN | ISIG | ECHO);
    tcsetattr(0, TCSANOW, &termios);
    tcflush(0, TCIOFLUSH);

    println!("-----> {:?}", term_orig);
    print!(">> ");
    io::stdout().flush();
    for byte in io::stdin().bytes() {
        let b = byte.unwrap();
        let mut command:String = String::new();
        if let Ok(string) = String::from_utf8(buf.clone()) {
            command = string;
        }


        match b {
            3 => break,
            9 => {
                //Tab
                let mut outbuf:String = String::new();
                let res = cli.complete(&command);
                
                for suggestion in res.iter() {
                    outbuf.push_str(suggestion);
                    outbuf.push_str("  ");
                }

                //TODO: meh...
                match command.chars().last() {
                    Some(' ') => {
                        if res.len() == 1 {
                            buf.extend(res[0].bytes());
                            command.push_str(res[0]);
                        }
                    }
                    _ => {
                        if res.len() > 1 {
                        } else if res.len() == 1 {
                        } else {
                            //nothing found
                        }
                    }
                }
       
                println!("\n{}", outbuf);
                print!(">> {}", command);
                io::stdout().flush();
            }
            10 => {
                //CRLF
                println!("\nexecute for: '{}'", command);
                cli.exec(&command);
                command.clear();
                buf.clear();
                print!(">> ");
                io::stdout().flush();
            }
            0x7f => {
                //backspace
                print!("{} {}", 0x08 as char, 0x08 as char);
                io::stdout().flush();
                buf.pop();
            }
            _ => {
                buf.push(b);
                print!("{}", b as char);
                io::stdout().flush();
            }
        }
    }

    tcsetattr(0, termios::TCSANOW, &term_orig);
}
