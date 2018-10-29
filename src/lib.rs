extern crate cqpsdk;
extern crate encoding;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
extern crate url;

use cqpsdk::cqpapi;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::all::GBK;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::sync::RwLock;
use url::Url;

macro_rules! gbk {
	($x: expr) => (CString::new(GBK.encode($x, EncoderTrap::Ignore).unwrap()).unwrap().into_raw());
}

macro_rules! utf8 {
	($x: expr) => (&GBK.decode(CStr::from_ptr($x).to_bytes(), DecoderTrap::Ignore).unwrap()[..]);
}

lazy_static! {
    static ref AUTH_CODE: RwLock<i32> = RwLock::new(0);
}

#[export_name = "AppInfo"]
pub extern "stdcall" fn app_info() -> *const i8 {
    return gbk!("9,com.github.ustc_zzzz.table_tennis");
}

#[export_name = "Initialize"]
pub extern "stdcall" fn initialize(auth_code: i32) -> i32 {
    *AUTH_CODE.write().unwrap() = auth_code;
    return cqpapi::EVENT_IGNORE;
}

#[allow(unused)]
#[export_name = "PrivateMessageHandler"]
pub extern "stdcall" fn private_message_handler(sub_type: i32,
                                                send_time: i32,
                                                qq_num: i64,
                                                msg: *const i8,
                                                font: i32) -> i32 {
    unsafe {
        let auth_code = *AUTH_CODE.read().unwrap();
        let message: String = utf8!(msg).to_owned();

        // ping pong

        for pong in ping_to_pong(&message).iter() {
            cqpapi::CQ_sendPrivateMsg(auth_code, qq_num, gbk!(&pong));
        }
    }
    cqpapi::EVENT_IGNORE
}

#[allow(unused)]
#[export_name = "GroupMessageHandler"]
pub extern "stdcall" fn group_message_handler(sub_type: i32,
                                              send_time: i32,
                                              group_num: i64,
                                              qq_num: i64,
                                              anonymous_name: *const i8,
                                              msg: *const i8,
                                              font: i32) -> i32 {
    unsafe {
        let auth_code = *AUTH_CODE.read().unwrap();
        let message: String = utf8!(msg).to_owned();

        // ping pong

        for pong in ping_to_pong(&message).iter() {
            cqpapi::CQ_sendGroupMsg(auth_code, group_num, gbk!(&pong));
        }

        // ore result

        if group_num == 613604130 {
            let regex = Regex::new("ore(:|：)\\s*(.+)").unwrap();
            if let Some(par) = regex.captures(&message).and_then(|c| c.get(2)).map(|m| m.as_str()) {
                let text = format!("正在从 Ore 上检索 {} 的相关信息，请稍安勿躁。", par);
                cqpapi::CQ_sendGroupMsg(auth_code, group_num, gbk!(&text));
                let text = fetch_ore_result(par);
                cqpapi::CQ_sendGroupMsg(auth_code, group_num, gbk!(&text));
            }
        }
    }
    cqpapi::EVENT_IGNORE
}

fn ping_to_pong(ping: &str) -> Vec<String> {
    let mut count = 0;
    let mut ping_mut = ping.to_owned();
    let mut pongs = Vec::<String>::new();

    while ping_mut.contains("ping") && count < 4 {
        ping_mut = ping_mut.replacen("ping", "pong", 1);
        pongs.push(ping_mut.clone());
        count += 1;
    }

    if ping_mut.contains("ping") {
        pongs.push("Exception in thread \"main\": java.lang.StackOverflowError".to_owned());
    }

    pongs
}

fn fetch_ore_result(par: &str) -> String {
    let mut url = Url::parse("https://ore.spongepowered.org/api/v1/projects").unwrap();
    url.query_pairs_mut().append_pair("q", &par.replace("ljyys", "yinyangshi")).append_pair("sort", "1");

    let get_res = || Client::new().get(url.clone()).send().and_then(|mut res| res.json::<Vec<Value>>()).ok();
    let res_option = None.or_else(&get_res).or_else(&get_res).or_else(&get_res);

    res_option.map_or("网络是不是出问题了？".to_owned(), |list| if let Some(Value::Object(map)) = list.get(0) {
        let owner = map.get("owner")
            .and_then(|v| v.as_str())
            .map(|v| v.replace("yinyangshi", "ljyys"))
            .unwrap_or("unknown".to_owned());
        let name = map.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let date = map.get("recommended")
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("createdAt"))
            .and_then(|v| v.as_str())
            .map(|v| &v[0..10])
            .unwrap_or("unknown");
        let version = map.get("recommended")
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let link = map.get("recommended")
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("href"))
            .and_then(|v| v.as_str())
            .map(|v| format!("https://ore.spongepowered.org{}", v))
            .unwrap_or("unknown".to_owned());
        format!("找到和 {} 有关的插件啦！\n插件名称：{}\n插件作者：{}\n推荐版本：{}\n更新日期：{}\n下载链接：{}",
                par, name, owner, version, date, link)
    } else {
        format!("没有在 Ore 平台找到和 {} 有关的插件。", par)
    })
}
