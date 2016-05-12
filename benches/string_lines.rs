#![feature(test)]
extern crate test;
extern crate string_lines;
use string_lines::StringLines;
use test::Bencher;
use std::fs::remove_file;


fn bench_push(b: &mut Bencher,count:usize) {
	let path = format!("target/bench_push_{}.bench",count);
	let _ = remove_file(&path);
	let mut lines = StringLines::open(path).expect("Unable to open file");
	let mut base_line = String::new();
	for _ in 0..count {
		base_line.push('l');
	}
	let mut i = 0;
	b.iter(|| {
		i = i + 1;
    	let _ = lines.push(&format!("{} {}",base_line,i)).expect("Unable to push line");
	});	
}

fn bench_pop(b: &mut Bencher,count:usize) {
	let path = format!("target/bench_pop_{}.bench",count);
	let _ = remove_file(&path);
	let mut lines = StringLines::open(path).expect("Unable to open file");
	let mut base_line = String::new();
	for _ in 0..count {
		base_line.push('l');
	}
	for i in 0..10000 {
		let _ = lines.push(&format!("{} {}",base_line,i)).expect("Unable to push line");			
	}
	b.iter(|| {
    	let _ = lines.pop().expect("Unable to pop line");
	});	
}

#[bench]
fn bench_push_4(b: &mut Bencher) {
	bench_push(b,4);
}

#[bench]
fn bench_push_40(b: &mut Bencher) {
	bench_push(b,40);
}

#[bench]
fn bench_push_400(b: &mut Bencher) {
	bench_push(b,400);
}

#[bench]
fn bench_push_4000(b: &mut Bencher) {
	bench_push(b,4000);
}

#[bench]
fn bench_pop_4(b: &mut Bencher) {
	bench_pop(b,4);
}

#[bench]
fn bench_pop_40(b: &mut Bencher) {
	bench_pop(b,40);
}

#[bench]
fn bench_pop_400(b: &mut Bencher) {
	bench_pop(b,400);
}

#[bench]
fn bench_pop_4000(b: &mut Bencher) {
	bench_pop(b,4000);
}