#![feature(let_chains)]

use std::{
	fs::read_to_string,
	thread,
	net::TcpStream,
};
use json::{ parse, JsonValue };

#[path = "./core.rs"] mod core;
#[path = "./loops.rs"] mod loops;

use xrdCommon::HostInfo;

fn main() {
	let cnf = parse(&read_to_string("./cnf.json").unwrap()).unwrap();
	println!("xrd v{} ({})",&cnf["Version"][0],&cnf["Version"][1]);
	thread::scope(|s| {
		for vh in cnf["VirtualHosts"].members() {
			let virthost: &JsonValue = &vh;
			s.spawn(move || {
				let (hostinfo,conn) = core::connectToServer(virthost);
				loops::servLoop(hostinfo,&conn);
			});
		}
	});
	loop {}
}
