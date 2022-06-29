use std::{
    str::from_utf8,
    io::{ Read, Write },
    time::Duration,
    sync::{ Arc, Mutex },
    thread::sleep,
    mem::transmute,
    ops::Deref,
};
use xrdCommon::{ Packet, ThreadParameters, splitArgs, send, read, parse, cram, xform };

#[no_mangle]
pub extern fn execute(args: ThreadParameters) {
    loop {
	let tx = args.sendQueue.clone();
	let rx = args.recvQueue.clone();

	let mut txu = tx.lock().unwrap();
	let mut rxu = rx.lock().unwrap();

	let data = rxu.pop(args.index);
	if data.is_none() { continue; };

	dbg!(data.unwrap().data);
    }
}
