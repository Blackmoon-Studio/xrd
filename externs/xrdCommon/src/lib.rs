#![feature(let_chains)]

use std::{
	net::TcpStream,
	io::{ Read, Write },
	str::from_utf8,
	any::Any,
	clone::Clone,
	sync::{ Arc, Mutex },
	time::Duration,
	thread::sleep,
	panic::catch_unwind,
};
use crossbeam_deque::{
    Injector,
    Steal,
    Steal::{ Empty, Retry, Success },
};
use aho_corasick::AhoCorasick;

pub struct InjectorLock<T: Clone> {
	inj: Injector<Packet<T>>,
	tc: u8,
}

impl<T: Clone> InjectorLock<T> {
    	pub fn new(inj: Injector<Packet<T>>, tc: u8) -> Self {
		Self { inj: inj, tc: tc }
    	}
	pub fn pop(&mut self, index: u8) -> Option<Packet<T>> {
		let x = self.inj.steal();
		if x.is_success() {
			let y = x.success();
			let mut z = y.unwrap();
			if z.lim != index {
				self.inj.push(z.clone());
				return Some(z);
			} return Some(z);
		} else { return None; }
	} pub fn push(&mut self, toPush: Packet<T>) {
    		self.inj.push(toPush);
	}
}

#[derive(Clone,Debug)]
pub struct Packet<T> where T: Clone {
	pub lim: u8,
	pub data: T,
}

#[derive(Debug,Clone)]
pub struct HostInfo {
    pub prefix: String,
    pub admins: Vec<String>,
}

pub struct Uplink {
	stream: TcpStream,
	threadId: u8,
	cache: Arc<Mutex<InjectorLock<String>>>,
}

impl Clone for Uplink {
    fn clone(&self) -> Uplink {
	Uplink { stream: self.stream.try_clone().unwrap(), threadId: self.threadId.clone(), cache: self.cache.clone() }
    }
}

impl Uplink {
	pub fn new(c: TcpStream, threadId: u8, a: Arc<Mutex<InjectorLock<String>>>) -> Self {
    		Self { stream: c, threadId: threadId, cache: a }
	} pub fn receive(&mut self) -> String {
    		let data = read(self.stream.try_clone().unwrap());
    		let x = self.cache.clone();
    		let mut y = x.lock().unwrap();
    		y.push(Packet{lim:self.threadId,data:data.clone()});
    		data
	} pub fn fetchLast(&mut self) -> Option<String> {
    		let x = self.cache.clone();
    		let mut y = x.lock().unwrap();
		let data = y.pop(self.threadId);
		if data.is_none() { return None; }
    		Some(data.unwrap().data)
	} pub fn transmit(&mut self, data: Vec<u8>) {
		send(self.stream.try_clone().unwrap(),data);
	}
}

pub struct ThreadParameters {
	pub info: HostInfo,
	pub index: u8,
    	pub sendQueue: Arc<Mutex<InjectorLock<Vec<u8>>>>,
        pub recvQueue: Arc<Mutex<InjectorLock<String>>>,
}

pub fn recvWorkerLoop(i: Arc<Mutex<InjectorLock<String>>>, t: u8, mut c: Uplink) {
    loop {
	let mut x = c.receive();
	if x == "None" {
		let y = c.fetchLast();
		if y.is_none() { continue; }
    		x = y.unwrap();
	}
    	if x == "None" { continue; }
	let i = i.clone();
	i.lock().unwrap().push(Packet{lim:t,data:x});
    }
}

pub fn sendWorkerLoop(i: Arc<Mutex<InjectorLock<Vec<u8>>>>, index: u8, mut c: Uplink) {
	loop {
    		let i = i.clone();
		let x = i.lock().unwrap().pop(index);
    		if x.is_some() {
			let y = x.unwrap();
			c.transmit(y.data);
    		} else { continue; }
	}
}

pub fn splitArgs(prefix: &String, message: &Vec<String>) -> Vec<String> {
	let mut args = message[2].split(prefix).collect::<Vec<&str>>()[1].to_string();
	let mut argv = args.split(" ").map(|x| x.to_string()).collect::<Vec<String>>();
	argv
}

pub fn read(mut conn: TcpStream) -> String {
	let mut parseBuf = [0;1024];
	conn.read(&mut parseBuf);
	let mut parsed = if from_utf8(&parseBuf).is_ok() { from_utf8(&parseBuf).unwrap() } else { "None" }.to_string();
	parsed.retain(|c| "\nABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.?-=\"\'<>1234567890 /".chars().collect::<Vec<char>>().contains(&c));
	if parsed.chars().collect::<Vec<char>>().len() < 16 { parsed = "None".to_string(); };
	parsed
}

pub fn parse(message: String) -> Vec<String> {
	let resFunc = catch_unwind(|| {
		let message = if message.strip_prefix("\\\u{1}\0\0\u{2}\0\0\0").is_some() { message.strip_prefix("\\\u{1}\0\0\u{2}\0\0\0").unwrap().to_string() } else { message };
		let mut xmllist: Vec<&str> = message.split("\n").collect();
		let ac_types = AhoCorasick::new_auto_configured(&["<string>","<boolean>","<i4>","<struct>"]);
		let ac_endtypes = AhoCorasick::new_auto_configured(&["</string>","</boolean>","</i4>","</struct>"]);
		let mut methodName = String::new();
		let mut argv = Vec::new();
		if xmllist[0].starts_with(r#"<?xml version="1.0" encoding="UTF-8"?><methodResponse>"#) {
			return(vec!["Invalid XML-RPC".to_string()]);
		} if xmllist[0].starts_with("<?xml") {
			if xmllist[1].starts_with("<methodCall>") {
				for x in &xmllist[2..] {
					if x.starts_with("<methodName>") {
						methodName.push_str(&x[x.find('>').unwrap()+1..x.rfind('<').unwrap()]);
					} else if x.starts_with("<params>") { continue; }
					if x.contains("<value>") || x.starts_with("<param><value>") {
						let mat = if ac_types.find(x).is_some() { ac_types.find(x).unwrap() } else { continue; };
						let mat2 = if ac_endtypes.find(x).is_some() { ac_endtypes.find(x).unwrap() } else { continue; };
						argv.push(String::from(&x[mat.end()..mat2.start()]));
					}
				}
			}
		}
		argv.push(methodName);
		argv
	});
	if resFunc.is_ok() {
		return resFunc.unwrap();
	} vec!["Invalid XML-RPC, check server output".to_string()]
}

pub fn xform(minified: String) -> String {
	let splits: Vec<&str> = minified[..].split("\n").collect();
	let mut lang: String = String::from("");
	let mut params: Vec<&str> = vec![];
	for str in splits {
		match str {
			"mc" => lang.push_str("<methodCall>"),
			"pa" => lang.push_str("<params>"),
			_ if str.starts_with("mn") => lang.push_str(&format!("<methodName>{}</methodName>",str.split("mn ").collect::<Vec<&str>>()[1])[..]),
			_ if str.starts_with("paa va") => params.push(str.split("paa va ").collect::<Vec<&str>>()[1]),
			_ => {},
		};
	} for str in params {
		match str {
			_ if str.starts_with("str") => lang.push_str(&format!("<param><value><string>{}</string></value></param>",&str.split("str ").collect::<Vec<&str>>()[1..][0])[..]),
			_ if str.starts_with("bool") && str.contains("true") => lang.push_str("<param><value><boolean>1</boolean></value></param>"),
			_ if str.starts_with("bool") && str.contains("false") => lang.push_str("<param><value><boolean>0</boolean></value></param>"),
			_ if str.starts_with("int") => lang.push_str(&format!("<param><value><i4>{}</i4></value></param>",&str.split("int ").collect::<Vec<&str>>()[1..][0])[..]),
			_ => {},
		};
	} lang.push_str("</params></methodCall>");
	String::from(lang)
}

pub fn cram(toCram: String) -> Vec<u8> {
	toCram[..].as_bytes().to_vec()
}

pub fn send(mut stream: TcpStream, toSend: Vec<u8>) {
	let handler = 0x80000000_u32.to_le_bytes();
	let len = toSend.len() as u32;
	let lenbytes = len.to_le_bytes();
	stream.write(&lenbytes);
	stream.write(&handler);
	stream.write(&toSend[..]);
}
