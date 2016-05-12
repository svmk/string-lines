extern crate file_lock;
use std::fs::{OpenOptions,File};
use std::io;
use std::io::{Seek,SeekFrom,Write,Read};
use std::fmt;
use std::path::Path;
use std::convert::AsRef;
use std::os::unix::io::AsRawFd;
use std::error;
use std::convert::From;
use std::result;
use file_lock::{Lock,AccessMode,LockKind};
use std::string::FromUtf8Error;
use std::iter::Extend;

#[derive(Debug)]
pub struct StringLines {
	file: File,
	lock: Lock,	
}

#[derive(Debug)]
pub enum Error {
	/// File error
	FileError(io::Error),
	/// Locking error
	LockError(file_lock::Error),
	/// UTF8 error
	Utf8Error(FromUtf8Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		match self {
			&Error::FileError(ref error) => {
				let _ = try!(write!(f,"File error: {}",error));
			},
			&Error::LockError(ref error) => {
				let _ = try!(write!(f,"Locking error: {:?}",error));
			},
			&Error::Utf8Error(ref error) => {
				let _ = try!(write!(f,"UTF8 error: {:?}",error));
			},
		}
		return Ok(());
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		match self {
			&Error::FileError(..) => {
				"File error"
			},
			&Error::LockError(..) => {
				"Locking error"
			},
			&Error::Utf8Error(..) => {
				"UTF8 error"
			},
		}
	}
}

impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		return Error::FileError(error);
	}
}

impl From<file_lock::Error> for Error {
	fn from(error: file_lock::Error) -> Self {
		return Error::LockError(error);
	}
}

impl From<FromUtf8Error> for Error {
	fn from(error: FromUtf8Error) -> Self {
		return Error::Utf8Error(error);
	}
}

pub type Result<T> = result::Result<T,Error>;

impl StringLines {
	/// Attempts to open a file in read-write mode.
	pub fn open<P: AsRef<Path>>(path: P) -> Result<StringLines> {
		let mut options = OpenOptions::new();
		options.read(true);
		options.write(true);
		options.create(true);
		let file = try!(options.open(path));
		let lock = Lock::new(file.as_raw_fd());
		Ok(StringLines {
			file: file,			
			lock: lock,			
		})
	}

	/// Appends an element to the back of a collection.
	pub fn push(&mut self,s:&str) -> Result<()> {
		let s = format!("{}\n",s);
		let _ = try!(self.lock.lock(LockKind::Blocking, AccessMode::Write));
		let _ = try!(self.file.seek(SeekFrom::End(0)));
		let _ = try!(self.file.write(s.as_bytes()));
		let _ = try!(self.lock.unlock());
		return Ok(());
	}

	fn pop_inner(&mut self,mut offset:i64,mut data:Vec<u8>) -> Result<Option<String>> {
		if offset <= 0 {
			offset = 0;
		}
		let _ = try!(self.file.set_len(offset as u64));
		let _ = try!(self.lock.unlock());		
		if data.len() > 0 {			
			data.reverse();
			let result = try!(String::from_utf8(data));
			return Ok(Some(result));
		} else {
			return Ok(None);
		}
	}

	/// Removes the last element from a collection and returns it, or None if it is empty.
	pub fn pop(&mut self) -> Result<Option<String>> {
		let _ = try!(self.lock.lock(LockKind::Blocking, AccessMode::Write));
		let len = try!(self.file.metadata()).len() as i64;
		let mut offset: i64 = len - 1024;		
		let mut data: Vec<u8> = Vec::with_capacity(1024);
		if offset < 0 {
			offset = 0;
		}
		loop {			
			let _ = try!(self.file.seek(SeekFrom::Start(offset as u64)));
			let mut buf = [0; 1024];
			match self.file.read(&mut buf) {
				Ok(0) => {
					break;
				}
				Ok(readed) => {
					let buf = &buf[..readed];					
					let mut lines: Vec<&[u8]> = buf.split(|c| { *c == 0x0A}).collect();					
					let lines_len = lines.len() as i32;
					if lines_len >= 2 {
						let mut char_offset: i64 = 0;
						for _ in 0..lines_len - 1 {							
							if let Some(last_line) = lines.pop() {
								char_offset = char_offset + 1;
								if last_line.len() >= 1 {
									data.extend(last_line.iter().rev());	
									return self.pop_inner(
										len - data.len() as i64 - char_offset,
										data
									);
								}
							}
						}
					}
					offset = offset - buf.len() as i64;
					data.extend(buf.iter().rev());	
				},
				Err(error) => {
					let _ = try!(self.lock.unlock());
					return Err(Error::from(error));
				},
			}
			if offset < 0 {
				break;
			}		
		}		
		return self.pop_inner(offset,data);			
	}

	/// Returns the number of elements in the collection.
	pub fn clear(&mut self) -> Result<()> {
		let _ = try!(self.lock.lock(LockKind::Blocking, AccessMode::Write));
		let _ = try!(self.file.set_len(0));
		let _ = try!(self.lock.unlock());		
		return Ok(());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs::remove_file;

    #[test]
    fn test_push_pop() {
    	let path = "target/test_push_pop.test";
    	let _ = remove_file(&path);
    	let mut lines = StringLines::open(path).expect("Unable to open file");
    	let mut items = vec![];
    	for i in 1..101 {    	
	    	let line = format!("line {}",i);	
	    	let _ = lines.push(&line).expect("Unable to push line");
	    	items.push(line);
    	}
    	loop {
    		match items.pop() {
    			Some(item_line) => {
    				let line = lines.pop().expect("Unable to pop line");    				
		    		assert_eq!(Some(item_line), line);
    			},
    			None => {
    				break;
    			},
    		}
    	}
    	let line = lines.pop().expect("Unable to pop line");
    	assert_eq!(line, None);
    	let line = lines.pop().expect("Unable to pop line");
    	assert_eq!(line, None);
    	let line = lines.pop().expect("Unable to pop line");
    	assert_eq!(line, None);
    }

    #[test]
    fn test_len() {
    	let path = "target/test_len.test";
    	let _ = remove_file(&path);
    	let _ = remove_file(&path);
    	let mut lines = StringLines::open(path).expect("Unable to open file");
    	let mut items = vec![];
    	for i in 1..101 {    	
	    	let line = format!("line {}",i);	
	    	let _ = lines.push(&line).expect("Unable to push line");
	    	items.push(line);
    	}
    	lines.clear().expect("Unable to clear collection");
    	let line = lines.pop().expect("Unable to pop line");
    	assert_eq!(line, None);
    	let line = lines.pop().expect("Unable to pop line");
    	assert_eq!(line, None);
    	let line = lines.pop().expect("Unable to pop line");
    	assert_eq!(line, None);
    }
}
