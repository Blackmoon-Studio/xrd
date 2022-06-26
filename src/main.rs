#![feature(let_else)]

use std::{
	fs::read_to_string,
	thread,
	net::TcpStream,
};
use json::{ parse, JsonValue };

#[path = "./core.rs"] mod core;
#[path = "./loops.rs"] mod loops;

fn main() {
	let cnf = parse(&read_to_string("./cnf.json").unwrap()).unwrap();
	println!("xrd v{} ({})",&cnf["Version"][0],&cnf["Version"][1]);
	thread::scope(|s| {
		for vh in cnf["VirtualHosts"].members() {
			let virthost: &JsonValue = &vh;
			s.spawn(|| {
				let (prefix,conn) = core::connectToServer(virthost);
				loops::servLoop(&prefix,&conn);
			});
		}
	});
	loop {}
}
