use std::{
	net::TcpStream,
	io::{ Read, Write },
	fs::{ read_dir, read_to_string },
	mem::transmute,
	str::from_utf8,
	thread,
};
use libloading::{ Library, Symbol };

use xrdCommon::{ Conn, send, parse, cram, xform };

pub fn servLoop<'a>(prefix: &'a String, conn: &'a TcpStream) {
	unsafe {
		thread::scope(|s| {
			for file in read_dir("./plugins").unwrap() {
				s.spawn(|| {
					let conn = transmute::<Conn<'a>,Conn<'static>>(Conn(conn));
					let lib = Library::new(file.unwrap().path()).unwrap();
					let func: Symbol<unsafe extern fn(String,Conn)> = lib.get(b"execute").unwrap();
					func(prefix.to_string(),conn);
				});
			}
		});
	}
}
