use std::{
    str::from_utf8,
    io::{ Read, Write },
};
use xrdCommon::{ Conn, send, read, parse, cram, xform };

#[no_mangle]
pub extern fn execute(prefix: String, conn: Conn) {
    loop {
        let mut parsev = parse(read(conn.0.try_clone().unwrap()));
        let head = parsev.pop().unwrap();
        if head == "TrackMania.PlayerChat" {
            if parsev[2].starts_with(&prefix) {
                let mut args = parsev[2].split("/").collect::<Vec<&str>>()[1].to_string();
                let mut argv = args.split(" ").collect::<Vec<&str>>();
                if argv[0] == "echo" {
                    argv.remove(0);
                    let mut echov = argv.join(" ");
                    send(conn.0.try_clone().unwrap(),cram(xform(format!("
mc
mn ChatSendToLogin
pa
paa va str {}
paa va str {}", echov, parsev[1]).to_string())));
                }
            }
        }
    }
}