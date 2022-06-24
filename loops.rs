use std::{
	net::TcpStream,
	io::{ Read, Write },
};

mod core;

use core::{ send, cram, xform };

pub fn servLoop(conn: TcpStream) {

}
