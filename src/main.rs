mod utils;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine as _;

use crate::utils::notify;

#[derive(Clone)]
struct CommentObject {
    comment: HashMap<String, String>,
    author: HashMap<String, String>
}

impl CommentObject {
    pub fn new(string: String) -> Self {
        let split: Vec<&str> = string.split(":").collect();

        CommentObject {
            comment: utils::format_kv(split[0].to_string(), "~"),
            author: utils::format_kv(split[1].to_string(), "~"),
        }
    }
}

fn contains_mention(comment: &String, aliases: &Vec<String>) -> bool {
    for alias in aliases {
        if comment.contains(alias) {
            return true;
        }
    }
    return false;
}

#[tokio::main]
#[allow(unused_assignments)]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let mut aliases: Vec<String> = vec![];
    let mut username = String::new();
    let mut debug_logs = false;

    for arg in &args {
        if arg.clone() == args[0] { continue; }

        if arg == "--debug-logs" {
            debug_logs = true;
        }

        if arg.starts_with("-username:") {
            if let Some(usr) = arg.strip_prefix("-username:") {
                username = usr.to_string();
            }
        } else {
            aliases.push(arg.to_string());
        }
    }

    if aliases.is_empty() {
        eprintln!("ERROR: You need to pass in some aliases!");
    }

    let daily_id = utils::get_special_id().await?;
    println!("Running on level ID {}", daily_id);
    println!("Aliases: {:?}", aliases);

    if !username.is_empty() {
        println!("Username (to prevent self mentions): {}", username);
    }

    println!("Debug logs: {}", if debug_logs { "ON" } else { "OFF" });

    tokio::spawn(async move {
        let mut previous_mentions: Vec<CommentObject> = vec![];

        loop {
            let req = utils::request_gd_servers(
                "getGJComments21.php",
                format!("levelID={}&secret=Wmfd2893gb7", daily_id).as_str()
            ).await;

            match req {
                Ok(res) => {
                    let comments: Vec<&str> = res.split("|").collect();

                    for comment in comments {
                        let mut obj = CommentObject::new(comment.to_string());
                        if debug_logs {
                            println!("Encoded: {}", &obj.comment["2"]);
                        }

                        let decoded_bytes_res = URL_SAFE.decode(&obj.comment["2"]);
                        if decoded_bytes_res.is_err() {
                            eprintln!("Failed to decode base64: {}", decoded_bytes_res.unwrap_err());
                            continue;
                        }
                        let decoded_bytes = decoded_bytes_res.unwrap();

                        let decoded = String::from_utf8_lossy(&decoded_bytes);

                        if debug_logs {
                            println!("Decoded: {}", decoded);
                        }

                        if contains_mention(&decoded.to_string(), &aliases) {
                            let mut is_previous = false;
                            for previous in &previous_mentions {
                                if previous.comment["6"] == obj.comment["6"] {
                                    is_previous = true;
                                    break;
                                }
                            }
                            if is_previous { continue; }
                            if obj.author["1"] == username { continue; }

                            obj.comment.insert("2".to_string(), decoded.to_string());
                            previous_mentions.push(obj.clone());
                            println!("FOUND MENTION by {}: {}", obj.author["1"], obj.comment["2"]);
                            notify(
                                format!("{} mentioned you!", obj.author["1"]),
                                obj.comment["2"].clone()
                            );
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Could not get comments: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    tokio::signal::ctrl_c().await?;

    println!("\nExiting...");
    Ok(())
}