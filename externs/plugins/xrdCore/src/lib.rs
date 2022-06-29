use std::{
    str::from_utf8,
    io::{ Read, Write },
    time::Duration,
    sync::{ Arc, Mutex },
    thread::sleep,
    mem::transmute,
    ops::Deref,
};
use xrdCommon::{ HostInfo, InjectorLock, Packet, splitArgs, send, read, parse, cram, xform };

#[no_mangle]
pub extern fn execute(info: HostInfo, index: u8, sendQueue: Arc<Mutex<InjectorLock<Vec<u8>>>>, recvQueue: Arc<Mutex<InjectorLock<String>>>) {
    loop {
        let i = sendQueue.clone();
        let mut sendQueue = i.lock().unwrap();
        let j = recvQueue.clone();
        let mut recvQueue = j.lock().unwrap();
        let mut steal = recvQueue.pop(index.clone());
        if steal.is_none() { continue; }
        let mut toParse = steal.unwrap();
        let mut parsev = parse(toParse.data.clone());
        let head = parsev.pop().unwrap();
        if head == "TrackMania.PlayerChat" {
            if parsev[2].starts_with(&info.prefix) {
                let mut argv = splitArgs(&info.prefix, &parsev);
		if argv[0].eq("kick") {
    			if info.admins.contains(&parsev[1]) {
				argv.remove(0);
				sendQueue.push(Packet{lim:index,data:cram(xform(format!("
mc
mn Kick
pa
paa va str {}
paa va str {}",argv[0],argv[1..].join(" "))))});
    			} else {
        			sendQueue.push(Packet{lim:index,data:cram(xform(format!("
mc
mn ChatSendToLogin
pa
paa va str {}
paa va str {}","You tried to use an admin-only command, but you have no permission.",parsev[1])))});
    			}
    		} else if argv[0].eq("ban") {
    			if info.admins.contains(&parsev[1]) {
				argv.remove(0);
				sendQueue.push(Packet{lim:index,data:cram(xform(format!("
mc
mn BanAndBlackList
pa
paa va str {}
paa va str {}
paa va bool true",argv[0],argv[1..].join(" "))))});
    			} else {
        			sendQueue.push(Packet{lim:index,data:cram(xform(format!("
mc
mn ChatSendToLogin
pa
paa va str {}
paa va str {}","You tried to use an admin-only command, but you have no permission.",parsev[1])))});
    			}
    		} else if argv[0].eq("corehelp") {
			let helpVec = vec![
    				"Help for xrdCore:",
    				"--ADMIN ONLY COMMANDS--",
    				"kick: Kicks a player with a specified reason.",
    				"ban: Bans a player with a specified reason.",
			];
			for x in helpVec {
        			sendQueue.push(Packet{lim:index,data:cram(xform(format!("
mc
mn ChatSendToLogin
pa
paa va str {}
paa va str {}",x,parsev[1])))});

			}
    		}
            }
        }
    } sleep(Duration::from_millis(10));
}
