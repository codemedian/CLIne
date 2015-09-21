extern crate cline;
extern crate termios;

use std::io;
use std::io::Read;
use std::os::unix::io::RawFd;
use termios::*;


fn main() {
    let mut cli = cline::Cli::new();
    let mut termios = Termios::from_fd(0).unwrap();
    let mut buf:Vec<u8> = Vec::new();
    let term_orig = termios;
    
    //termios.c_lflag = ECHONL;
    termios.c_lflag &= !(ICANON | IEXTEN | ISIG);
    tcsetattr(0, TCSANOW, &termios);

    for byte in io::stdin().bytes() {
        //println!("read {:?}", byte );
        let b = byte.unwrap();
        let mut command:String = String::new();
        buf.push(b);

        if let Ok(string) = String::from_utf8(buf.clone()) {
            //println!("--> {}", string);
            command = string;
        }

        match b {
            3 => break,
            9 => {
                //command.pop();
                println!("complete for: '{}'", command);
                println!("{:?}", cli.complete(&command));
            }
            10 => {
                command.pop();
                println!("execute for: '{}'", command);
                cli.exec(&command)
            }
            _ => {
                //println!("{:?}", b);
            }
        }


    }


    tcsetattr(0, termios::TCSANOW, &term_orig);
}
