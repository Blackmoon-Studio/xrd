use std::{
	net::TcpStream,
	io::{ Read, Write },
	str::from_utf8,
};
use json::JsonValue;
use byteorder::{ LittleEndian, WriteBytesExt };

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

pub fn send(mut stream: &TcpStream, toSend: Vec<u8>) {
	let handler = 0x80000000_u32.to_le_bytes();
	let len = toSend.len() as u32;
	let lenbytes = len.to_le_bytes();
	stream.write(&lenbytes);
	stream.write(&handler);
	stream.write(&toSend[..]);
}

pub fn connectToServer(vh: &JsonValue) -> TcpStream {
	let host = vh["Host"].as_str().expect("No host was specified.");
	let port = vh["Port"].as_str().expect("No port was specified.");
	let pref = vh["Prefix"].as_str().expect("No prefix was specified.");
	let typ = vh["Type"].as_str().expect("No server type was specified.");
	let pass = vh["Password"].as_str().expect("No SuperAdmin password was specified.");
	let adms = &vh["Admins"];
	
	let mut listener = TcpStream::connect(format!("{}:{}",host,port)).unwrap();

	listener.read(&mut [0;4]).unwrap();
	let mut ver = [0;12];
	listener.read(&mut ver).unwrap();
	
	assert_eq!(from_utf8(&ver).unwrap().trim_matches(char::from(0)),"GBXRemote 2");
	
	send(&listener,cram(xform(format!("
mc
mn Authenticate
pa
paa va str SuperAdmin
paa va str {}",pass))));
	send(&listener,cram(xform("
mc
mn EnableCallbacks
pa
paa va bool true".to_string())));
	listener
}
