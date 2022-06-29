use std::{
	net::TcpStream,
	io::{ Read, Write },
	str::from_utf8,
};
use json::JsonValue;
use xrdCommon::{ HostInfo, send, cram, xform };

pub fn connectToServer(vh: &JsonValue) -> (HostInfo,TcpStream) {
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
	
	send(listener.try_clone().unwrap(),cram(xform(format!("
mc
mn Authenticate
pa
paa va str SuperAdmin
paa va str {}",pass))));
	send(listener.try_clone().unwrap(),cram(xform("
mc
mn EnableCallbacks
pa
paa va bool true".to_string())));
	let hostinfo = HostInfo{ prefix: pref.to_string(), admins: adms.members().map(|x| { x.as_str().expect("Admins array is exclusively made of strings").to_string() }).collect::<Vec<String>>() };
	(hostinfo,listener)
}
