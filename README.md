# string-lines
[![Build Status](https://travis-ci.org/svmk/string-lines.svg?branch=master)](https://travis-ci.org/svmk/string-lines)
[![Latest Version](https://img.shields.io/crates/v/string-lines.svg)](https://crates.io/crates/string-lines)

[API Documentation](http://svmk.github.io/string-lines/0.1.0/string_lines/index.html)

Raw persistent database for storing string lines.

## Example

```rust
extern crate string_lines;
use string_lines::StringLines;
fn main() {	
	let mut lines = StringLines::open(
		"target/push_pop.example"
	).expect("Unable to open file");
	for i in 1..101 {      
		let line = format!("line {}",i);    
		let _ = lines.push(&line).expect("Unable to push line");
	}
	loop {
	    match lines.pop().expect("Unable to pop line") {
	        Some(line) => {
	            println!("{}",line);
	        },
	        None => {
	            break;
	        }
	    }
	}
}
```