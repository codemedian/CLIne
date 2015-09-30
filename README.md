# CLIne - a command line API for rust [![Build Status](https://travis-ci.org/chsitter/CLIne.svg)](https://travis-ci.org/chsitter/CLIne)
[Documentation](http://chsitter.github.io/CLIne/cline/)

The [`cline`](http://chsitter.github.io/CLIne/cline/) crate provides an API that allows users to register CLI commands with an execute and dynamic suggest callback to help implementing command line clients that support auto completion of commands


## Usage
``` rust
extern crate cline;

fn main() {
  let mut cli = cline::Cli::new();
  
  cli.register(vec!["list", "files"], | args | { /* this gets called on execute */ });
  
  cli.complete("li"); // returns vec!["list"]
  
  cli.exec("list files"); 
}
```

## Contributors
* [chsitter](https://github.com/chsitter/)

## License
Copyright © 2015 Christoph Sitter

Distributed under the [Apache License Version 2.0](LICENSE).
