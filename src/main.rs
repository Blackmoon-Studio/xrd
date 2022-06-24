use rayon::prelude::*;
use std::{
	fs::read_to_string,
};
use json::{ parse, JsonValue };

mod core;
mod loops;

fn main() {
	let cnf = parse(&read_to_string("cnf.json").unwrap()).unwrap();
	println!("xrd v{} ({})",&cnf["Version"][0],&cnf["Version"][1]);
	for vh in cnf["VirtualHosts"].members() {
		rayon::scope(|s| {
			let virthost: &JsonValue = &vh;
			s.spawn(move |_| {
				let conn = core::connectToServer(virthost);
				loops::servLoop(conn);
			});
		});
	}
}
