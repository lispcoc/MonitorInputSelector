#![windows_subsystem="windows"]
extern crate ddc;
extern crate serde_derive;
extern crate serde_json;

use std::fs::File;
use std::io::{Error, self, Read};

use regex::Regex;
use serde::{Deserialize, Serialize};
use ddc::Ddc;
use ddc_winapi::Monitor;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Test { text: String },
    setVpc { display: u8, vcp: u8, value: u16 },
    getVpc { index: u8 },
    getCapabilities { callback: String },
}

pub fn v8_str(v: Vec<u8>) -> String {
    let mut str = "[".to_string();
    for mut i in v {
        str = format!("{}{},", str, i);
    }
    str = format!("{}]", str);
    return (str)
}

fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn add_double_quote(s: &str) -> String {
    format!("\"{}\"", s)
}

pub fn internal_set_vpc(display: u8, vcp: u8, value: u16) -> (){
    for mut ddc in Monitor::enumerate().unwrap() {
        ddc.set_vcp_feature(vcp, value);
        return();
    }
    return()
}

pub fn internal_get_vpc(index: u8) -> u16{
    for mut ddc in Monitor::enumerate().unwrap() {
        let retval = ddc.get_vcp_feature(index).unwrap();
        return(retval.value());
    }
    return(0)
}

pub fn internal_capabilities() -> String{
    let mut retval : String = "[".to_string();
    
    for mut ddc in Monitor::enumerate().unwrap() {
        let capabilities_v8 = ddc.capabilities_string().unwrap();
        let capabilities_str = String::from_utf8(capabilities_v8).unwrap();
        retval = format!("{}{},", retval.to_string(), add_double_quote(&capabilities_str));
    }
    retval = format!("{}]", retval.to_string());
    return(retval);
}

fn main() {

    let mut monitor: String = "".to_string();

    let mut HTML: String = default_HTML.to_string();
    
    match get_file_contents("../../src/test.html".to_string()) {
        Ok(contents) => {
            println!("OK:file_open ");
            println!("{}", contents);
            HTML = contents;
        }
        Err(e) => {
            println!("Error:{}", e);
            return();
        }
    }

    web_view::builder()
        .title("Hello world!")
        .content(web_view::Content::Html(HTML))
        .size(320, 240)
        .user_data(())
        .invoke_handler(|webview, arg| {
            use Cmd::*;
            println!("{}", arg);

            match serde_json::from_str(arg).unwrap() {
                Test { text } => println!("{}", text),
                getVpc { index } => {
                    internal_get_vpc(index);
                    ()
                },
                setVpc { display, vcp, value } => {
                    internal_set_vpc(display, vcp, value);
                    ()
                },
                getCapabilities { callback } => {
                    let ret = &internal_capabilities();
                    println!("{}", ret);
                    match webview.eval(&format!("{}({})", callback, ret)) {
                        Ok(s) => println!("OK"),
                        Err(err) => eprintln!("IO Error => {}", err),
                    }
                },
            };

            Ok(())
        })
        .run()
        .unwrap();
}

const default_HTML: &str = r#"<!DOCTYPE html>
<html>
    <body>
        <div id="display">0</div>
        <button onclick="test2({cmd: 'test', text: 'aaa'})">count</button>

        <script>
            var mailbox = 0;
            function test2(arg) {
                document.getElementById('display').innerHTML = JSON.stringify(arg);
                //external.invoke(JSON.stringify(arg));
            },
            function update_mailbox(mail) {
                mailbox = mail;
            }
        </script>
    </body>
</html>"#;
