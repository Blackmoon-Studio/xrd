use std::{
	net::TcpStream,
	io::{ Read, Write },
	str::from_utf8,
};
use aho_corasick::AhoCorasick;

pub fn parse(message: String) -> Vec<String> {
	let mut xmllist: Vec<&str> = vec![];
	if message.char_indices().next().eq(&Some((0, 'V'))) { 
		xmllist = message.strip_prefix("V").unwrap().split('\n').collect::<Vec<&str>>();
	} else {
		xmllist = message.split('\n').collect::<Vec<&str>>();
	} let ac_types = AhoCorasick::new_auto_configured(&["<string>","<boolean>","<i4>","<struct>"]);
	let ac_endtypes = AhoCorasick::new_auto_configured(&["</string>","</boolean>","</i4>","</struct>"]);
	let mut methodName = String::new();
	let mut argv = Vec::new();
	if xmllist[0].starts_with("<?xml") {
		if xmllist[1].starts_with("<methodCall>") || xmllist[1].starts_with("<methodResponse>") {
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
		} else { return vec![String::from("Invalid XML-RPC, check server output")]; }
	} else { return vec![String::from("Invalid XML-RPC, check server output")]; }
	argv.push(methodName);
	argv
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
