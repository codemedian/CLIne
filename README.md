# CLIne - a command line API for rust [![Build Status](https://travis-ci.org/chsitter/CLIne.svg)](https://travis-ci.org/chsitter/CLIne)
[Documentation](http://chsitter.github.io/CLIne/cline/)

The [`cline`](http://chsitter.github.io/CLIne/cline/) crate provides an API that allows users to register CLI commands with an execute and dynamic suggest callback to help implementing command line clients that support auto completion of commands


## Usage
``` rust
extern crate cline;
use cline::{Cli, cline_run};

fn main() {
    let mut cli = Cli::new();

    cli.register(vec!["foo", "bar"], | _ | { println!("running foo bar") });
    cli.register(vec!["foo", "baz"], | _ | { println!("running foo baz") });

    cline_run(&mut cli);
}
```

## Contributors
* [chsitter](https://github.com/chsitter/)

## License
Copyright © 2015 Christoph Sitter

Distributed under the [Apache License Version 2.0](LICENSE).
