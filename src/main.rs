extern crate cline;
extern crate termios;

use std::io;
use std::io::prelude::*;
use std::os::unix::io::RawFd;
use termios::*;


fn main() {
    let mut cli = cline::Cli::new();
    let mut termios = Termios::from_fd(0).unwrap();
    let mut buf:Vec<u8> = Vec::new();
    let term_orig = termios;

    cli.register(vec!["show", "stuff"], | _ | { println!("wooo haaa") });
    
    termios.c_lflag = ECHONL;
    //termios.c_lflag &= !(ICANON | IEXTEN | ISIG);
    tcsetattr(0, TCSANOW, &termios);
    tcflush(0, TCIOFLUSH);

    for byte in io::stdin().bytes() {
        //println!("read {:?}", byte );
        let b = byte.unwrap();
        let mut command:String = String::new();
        if let Ok(string) = String::from_utf8(buf.clone()) {
            command = string;
        }


        match b {
            3 => break,
            9 => {
                let res = cli.complete(&command);
                match command.chars().last() {
                    Some(' ') => {
                        if res.len() > 0 {
                            buf.extend(res[0].bytes());
                            command.push_str(res[0]);
                        }
                    }
                    _ => {
                        command.clear();
                        buf.extend(res[0].bytes());
                        command.push_str(res[0]);
                    }
                }

                print!("{}", command);
                io::stdout().flush();
            }
            10 => {
                println!("execute for: '{}'", command);
                cli.exec(&command);
                print!("{}", command);
                io::stdout().flush();
            }
            _ => {
                buf.push(b);
            }
        }


    }


    tcsetattr(0, termios::TCSANOW, &term_orig);
}
