#![feature(let_chains)]

use std::{
	fs::{ read_to_string, read_dir },
	thread,
	net::TcpStream,
	sync::{ Arc, Mutex },
};
use json::{ parse, JsonValue };
use crossbeam_deque::{ Injector };
use libloading::{ Library, Symbol };

#[path = "./core.rs"] mod core;

use xrdCommon::{ HostInfo, InjectorLock, Packet, Uplink, ThreadParameters, recvWorkerLoop, sendWorkerLoop };

fn main() {
	let cnf = parse(&read_to_string("./cnf.json").unwrap()).unwrap();
	println!("xrd v{} ({})",&cnf["Version"][0],&cnf["Version"][1]);
	thread::scope(|s| {
    		let membs = cnf["VirtualHosts"].members().len() as u8;
    		let mut ct = 0;

		for vh in cnf["VirtualHosts"].members() {
			let txinj: Injector<Packet<Vec<u8>>> = Injector::new();
			let rxinj: Injector<Packet<String>> = Injector::new();
			let mut tx: Arc<Mutex<InjectorLock<Vec<u8>>>> = Arc::new(Mutex::new(InjectorLock::new(txinj, membs.to_owned())));
			let mut rx: Arc<Mutex<InjectorLock<String>>> = Arc::new(Mutex::new(InjectorLock::new(rxinj, membs.to_owned())));
			let crxinj: Injector<Packet<String>> = Injector::new();
			let mut crx: Arc<Mutex<InjectorLock<String>>> = Arc::new(Mutex::new(InjectorLock::new(crxinj, membs.to_owned())));
			let virthost: &JsonValue = &vh;
			s.spawn(move || {
				let (hostinfo,conn) = core::connectToServer(virthost);

				unsafe {
    					let mut index = 0;
    					for file in read_dir("./plugins").unwrap() {
        					let cnc = conn.try_clone().unwrap();
        					let ul = Uplink::new(cnc, index, crx.clone());

						let txc = tx.clone();
						let inc = index.clone();
						let ulc = ul.clone();
						s.spawn(move || {
							sendWorkerLoop(txc,inc,ulc);
						});

						let rxc = rx.clone();
						let inc = index.clone();
						let ulc = ul.clone();
						s.spawn(move || {
							recvWorkerLoop(rxc,inc,ulc);
						});

						let hic = hostinfo.clone();
						let inc = index.clone();
						let txc = tx.clone();
						let rxc = rx.clone();
						s.spawn(move || {
							let lib = Library::new(file.unwrap().path()).unwrap();
							let func: Symbol<unsafe extern fn(ThreadParameters)> = lib.get(b"execute").unwrap();
							func(ThreadParameters{info:hic,index:inc,sendQueue:txc,recvQueue:rxc});
						});

						index = index + 1;
    					}
				}
			}); ct = ct + 1;
		}
	});
	loop {}
}
