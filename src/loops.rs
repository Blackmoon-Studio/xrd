use std::{
	net::TcpStream,
	io::{ Read, Write },
	sync::{ Arc, Mutex },
	fs::{ read_dir, read_to_string },
	mem::transmute,
	boxed::Box,
	ops::Deref,
	str::from_utf8,
	thread,
};
use libloading::{ Library, Symbol };
use crossbeam_deque::{ Injector };

use xrdCommon::{ Conn, HostInfo, InjectorLock, Engine, Packet, sendWorkerLoop, recvWorkerLoop, send, parse, cram, xform };

pub fn servLoop<'a>(hostinfo: HostInfo, conn: &'a TcpStream) {
    	let hostinfo = Arc::new(Mutex::new(hostinfo));
	unsafe {
		thread::scope(|s| {
        		let txinj: Injector<Packet<Vec<u8>>> = Injector::new();
        		let rxinj: Injector<Packet<String>> = Injector::new();
        		let mut tx: Arc<Mutex<InjectorLock<Vec<u8>>>> = Arc::new(Mutex::new(InjectorLock::new(txinj)));
        		let mut rx: Arc<Mutex<InjectorLock<String>>> = Arc::new(Mutex::new(InjectorLock::new(rxinj)));
			for file in read_dir("./plugins").unwrap() {
    				let txclone1 = tx.clone();
    				let rxclone1 = rx.clone();
    				let txclone2 = tx.clone();
    				let rxclone2 = rx.clone();
    				let mut index = 0;
    				let indexClone = index.clone();
    				let clonedHostInfo = hostinfo.clone();
    				s.spawn(move || {
					sendWorkerLoop(txclone1,indexClone,conn.try_clone().unwrap());
    				}); s.spawn(move || {
					recvWorkerLoop(rxclone1,indexClone,conn.try_clone().unwrap());
        			}); s.spawn(move || {
    					let lock = clonedHostInfo.lock().unwrap();
    					let deref = lock.deref();
					let hostinfo = deref.to_owned();

					let lib = Library::new(file.unwrap().path()).unwrap();
					let func: Symbol<unsafe extern fn(HostInfo,u8,Arc<Mutex<InjectorLock<Vec<u8>>>>,Arc<Mutex<InjectorLock<String>>>)> = lib.get(b"execute").unwrap();

					func(hostinfo,indexClone,txclone2,rxclone2);
				}); index = index + 1;
			}
		});
	}
}
