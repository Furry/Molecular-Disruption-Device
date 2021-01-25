use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use dns_lookup::lookup_host;

use serde_json;
use traffic::udp;
use traffic::tcp;
use ws::WebsocketContext;
use reqwest::{self, blocking};

use crate::traffic;
use crate::coms::config_handler;

#[derive(Debug, Clone)]
pub struct CommandHandler {
    config: config_handler::Config_Handler
}

pub fn command_handler_new() -> CommandHandler {
    CommandHandler {
        config: config_handler::new()
    }
}

// I decieded to implement an actor into my command handler..
// Probably bad practice, but it gets the job done!
impl Actor for CommandHandler {
    type Context = ws::WebsocketContext<Self>;
}

// A handler for incoming websocket messages, where ctx is the messaging context
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CommandHandler {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Text(text)) => Self::call(self, text, ctx),
            _ => ()
        }
    }
}

impl CommandHandler {
    // Handle incoming websocket commands
    fn call(&mut self, content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        let args: Vec<String> = content.split(" ")
            .map(|x| x.to_string())
            .collect();
        let data = args.clone()
            .drain(1..)
            .as_slice()
            .join(" ");
        println!("{}", args[0]);

        let prep_args: Vec<String> = data.clone()
            .split(" ")
            .map(|x| String::from(x))
            .collect();

        // Bind certain commands to their functional counterparts
        match args[0].as_str() {
            "resolve" => Self::resolve(data, ctx),
            "echo" => Self::echo(data, ctx),
            "udp" => Self::udp_n(self, prep_args, ctx),
            "tcp" => Self::tcp_n(self, prep_args, ctx),
            "http" => Self::http(self, content, ctx),
            "help" => Self::help(content, ctx),
            "config" => Self::config(self, content, ctx),
            _ => Self::default(data, ctx)
        };
    }

    // Utility function for sending messages without excessive formatting
    pub fn send_message(content: &str, ctx: &mut WebsocketContext<CommandHandler>) {
        ctx.text(String::from(format!("[{{\"message\": \"{}\"}}]", content)))
    }

    pub fn send_error(content: &str, ctx: &mut WebsocketContext<CommandHandler>) {
        ctx.text(String::from(format!("[{{\"error\": \"{}\"}}]", content)))
    }

    pub fn send_raw(content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        ctx.text(content);
    }

    pub fn default(_content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        Self::send_error("Command is not known!", ctx)
    }

    pub fn help(_: String, ctx: &mut WebsocketContext<CommandHandler>) {
        Self::send_message("|nl|Type in a command for more information on it!|nl||nl|Network|nl|  - udp|nl|  - tcp|nl|  - http  - resolve|nl|Utility|nl|  - clr|nl|  - echo|nl| - config|nl||nl|", ctx)
    }

    pub fn echo(content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        Self::send_message(content.as_str(), ctx);
    }

    /// Resolves an input string
    pub fn resolve(content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        match lookup_host(content.as_str()) {
            Ok(ips) => {
                let mut string_ips: Vec<String> = Vec::new();
                for ip in ips {
                    string_ips.push(ip.to_string());
                };
                Self::send_raw(String::from(format!("[{{\"message_json\": {}}}]", serde_json::json!(string_ips).to_string())), ctx)
            },
            Err(_) => {
                Self::send_error("Null Resolution", ctx)
            }
        }
    }

    // This returns the entire body data, Add a way to trim it to show a %, or download it instead.
    // (A large message size often contains ' " ' escape characters)
    pub fn http(&self, content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        let args: Vec<&str> = content.split(" ").collect();
        let client = reqwest::blocking::Client::new();

        if args.len() > 2 {
            if args[2] == "flood" {
                let bldrwrapper: Result<blocking::RequestBuilder, ()> = match args[1] {
                    "get" => Ok(client.get(args[3])),
                    "post" => Ok(client.post(args[3])),
                    "put" => Ok(client.put(args[3])),
                    "patch" => Ok(client.patch(args[3])),
                    _ => Err(())
                };
                match bldrwrapper {
                    Ok(bldr) => {
                        let mut count: u64 = 0;
                        for _ in 0..args[4].parse::<u32>().unwrap() {
                            println!("for _ in 0..args[2]");
                            match bldr.try_clone().unwrap().send() {
                                Err(_) => (),
                                _ => count += 1
                            };
                        }
                        Self::send_message(format!("Successfully sent {} messages", count).as_str(), ctx);
                    },
                    Err(_) => Self::send_message(format!("'{}' is not a valid http method! Valid methods: get, post, put, patch", args[1]).as_str(), ctx)
                };
            } else {
                let bldr: Result<blocking::RequestBuilder, ()> = match args[1] {
                    "get" => Ok(client.get(args[2])),
                    "post" => Ok(client.post(args[2])),
                    "put" => Ok(client.put(args[2])),
                    "patch" => Ok(client.patch(args[2])),
                    _ => {
                        Err(())
                    }
                };
                match bldr {
                    Ok(bldr) => {
                        match bldr.send() {
                            Ok(resp) => {
                                Self::send_message(format!("Response: {}", resp.text().unwrap()).as_str(), ctx)
                            },
                            Err(err) => {
                                println!("{:?}", err);
                                Self::send_message("Request failed!", ctx)
                            }
                        }
                    },
                    Err(_) => { 
                        Self::send_message(format!("'{}' is not a valid http method! Valid methods: get, post, put, patch", args[1]).as_str(), ctx)
                    }
                }
            }
        } else {
            Self::send_message("Usage: http get/post/patch/put www.google.com", ctx)
        }
    }

    // probe http://localhost/ 50
    // flood http://localhost/ 50 5000
    pub fn udp_n(&mut self, args: Vec<String>, ctx: &mut WebsocketContext<CommandHandler>) {
        if args.len() >= 3 {
            let host = match lookup_host(args[1].as_str()) {
                Ok(ips) => {
                    ips[0].to_string()
                },
                Err(_) => {
                    return Self::send_message(format!("Could not resolve host of {}!", args[1]).as_str(), ctx)
                }
            };
            let final_addr = format!("{}:{}", host, args[2]);
            match args[0].as_str() {
                "probe" => {
                    match udp::construct(final_addr.clone()) {
                        Ok(mut sckt) => {
                            let status = sckt.send(self.config.payload.clone());
                            if status == true {
                                Self::send_message("Response!", ctx)
                            } else {
                                Self::send_message("No Response!", ctx)
                            }
                        }
                        Err(_) => {
                            Self::send_message(format!("Requested address {} not a valid socket.", final_addr).as_str(), ctx)
                        }
                    }
                },
                "flood" => {
                    let count = match args[3].parse::<u32>() {
                        Ok(num) => num,
                        Err(_) => return Self::send_error(format!("Could not parse {} to a valid number!", args[3]).as_str(), ctx)
                    };

                    if args.len() >= 4 {
                        match udp::construct(final_addr.clone()) {
                            Ok(sckt) => {
                                sckt.flood(self.config.payload.clone(), count);
                                Self::send_message(format!("Successfully sent {} messages", count).as_str(), ctx);
                            },
                            Err(_) => {
                                Self::send_message(format!("Requested address {} not a valid socket.", final_addr).as_str(), ctx)
                            }
                        }
                    } else {
                        Self::send_message("udp Usage:|nl|  - udp (flood)/probe addr port (count)|nl|  - udp probe 127.0.0.1 334|nl|  - udp flood 127.0.0.1 334 500", ctx)
                    }
                },
                _ => {
                    Self::send_message("udp Usage:|nl|  - udp (flood)/probe addr port (count)|nl|  - udp probe 127.0.0.1 334|nl|  - udp flood 127.0.0.1 334 500", ctx)
                }
            }
        } else {
            Self::send_message("udp Usage:|nl|  - udp (flood)/probe addr port (count)|nl|  - udp probe 127.0.0.1 334|nl|  - udp flood 127.0.0.1 334 500", ctx)
        }
    }

    // probe http://localhost/ 80
    // flood http://localhost/ 70 500
    pub fn tcp_n(&mut self, args: Vec<String>, ctx: &mut WebsocketContext<CommandHandler>) {
        if args.len() >= 3 {
            let host = match lookup_host(args[1].as_str()) {
                Ok(ips) => {
                    ips[0].to_string()
                },
                Err(_) => {
                    return Self::send_message(format!("Could not resolve host of {}!", args[1]).as_str(), ctx)
                }
            };
            let final_addr = format!("{}:{}", host, args[2]);
            match args[0].as_str() {
                "probe" => {
                    match tcp::construct(final_addr.clone()) {
                        Ok(mut sckt) => {
                            let status = sckt.send(self.config.payload.clone());
                            if status == true {
                                Self::send_message("Response!", ctx)
                            } else {
                                Self::send_message("No Response!", ctx)
                            }
                        }
                        Err(_) => {
                            Self::send_message(format!("Requested address {} not a valid socket.", final_addr).as_str(), ctx)
                        }
                    }
                },
                "flood" => {
                    let count = match args[3].parse::<u32>() {
                        Ok(num) => num,
                        Err(_) => return Self::send_error(format!("Could not parse {} to a valid number!", args[3]).as_str(), ctx)
                    };

                    if args.len() >= 4 {
                        match tcp::construct(final_addr.clone()) {
                            Ok(sckt) => {
                                sckt.flood(self.config.payload.clone(), count);
                                Self::send_message(format!("Successfully sent {} messages", count).as_str(), ctx);
                            },
                            Err(_) => {
                                Self::send_message(format!("Requested address {} not a valid socket.", final_addr).as_str(), ctx)
                            }
                        }
                    } else {
                        Self::send_message("tcp Usage:|nl|  - tcp flood/(probe) addr port (count)|nl|  - tcp probe 127.0.0.1 333|nl|  - tcp flood 127.0.0.1 333 500", ctx)
                    }
                },
                _ => {
                    Self::send_message("tcp Usage:|nl|  - tcp flood/(probe) addr port (count)|nl|  - tcp probe 127.0.0.1 333|nl|  - tcp flood 127.0.0.1 333 500", ctx)
                }
            }
        } else {
            Self::send_message("tcp Usage:|nl|  - tcp flood/(probe) addr port (count)|nl|  - tcp probe 127.0.0.1 333|nl|  - tcp flood 127.0.0.1 333 500", ctx)
        }
    }

    ///////////////////////////
    // THIS CODE IS REDACTED //
    ///////////////////////////
    pub fn tcp(&self, content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        let args: Vec<&str> = content.split(" ").collect();

        if args.len() > 1 {
            match args[1] {
                "probe" => {
                    if args.len() >= 3 {
                        match lookup_host(args[2]) {
                            Ok(ips) => {
                                println!("{}:{}", ips[0], args[3]);
                                match tcp::construct(format!("{}:{}", ips[0], args[3]).to_string()) {
                                    Ok(mut sckt) => {
                                        let status = sckt.send(self.config.payload.clone());
                                        if status == true {
                                            Self::send_message("Response!", ctx)
                                        } else {
                                            Self::send_message("No Response!", ctx)
                                        }
                                    },
                                    Err(_) => {
                                        Self::send_message(format!("Requested address {}:{} is not a valid socket.", ips[0], ips[3]).as_str(), ctx)
                                    }
                                }
                            },
                            Err(_) => {
                                Self::send_message(format!("Could not resolve host of {}!", args[0]).as_str(), ctx)
                            }
                        }
                    } else {
                        Self::send_message("Usage: tcp probe 127.0.0.1 40|nl|    - Sends a packet to the ip on port 40, returning a success or failure message.", ctx)
                    }
                },
                "flood" => {
                    if args.len() >= 4 {
                        match lookup_host(args[2]) {
                            Ok(ips) => {
                                match tcp::construct(format!("{}:{}", ips[0], args[3]).to_string()) {
                                    Ok(sckt) => {
                                        let count = args[4].parse::<u32>().unwrap();
                                        let data = args.clone().drain(4..).as_slice().join("");
                                        sckt.flood(data, count);
                                    },
                                    Err(_) =>{ 
                                        Self::send_message(format!("Requested address {}:{} is not a valid socket.", ips[0], ips[3]).as_str(), ctx)
                                    }
                                }
                            },
                            Err(_) => {
                                Self::send_message(format!("Could not resolve host of {}!", args[0]).as_str(), ctx)
                            }
                        }
                    }
                }
                _ => {
                    Self::send_message(format!("tcp => {}; {} Unknown Command!", args[1], args[1]).as_str(), ctx)
                }
            }
        } else {
            Self::send_message(format!("Sub-Commands:|nl|  - probe|nl|  - flood").as_str(), ctx)
        }
    }

    pub fn udp(&self, content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        let args: Vec<&str> = content.split(" ").collect();
        if args.len() > 1 {
            match args[1] {
                "probe" => {
                    if args.len() >= 3 {
                        match lookup_host(args[2]) {
                            Ok(ips) => {
                                println!("{}:{}", ips[0], args[3]);
                                match udp::construct(format!("{}:{}", ips[0], args[3]).to_string()) {
                                    Ok(mut sckt) => {
                                        let status = sckt.send("probe".to_string());
                                        if status == true {
                                            Self::send_message("Response!", ctx)
                                        } else {
                                            Self::send_message("No Response!", ctx)
                                        }
                                    },
                                    Err(_) => {
                                        Self::send_message(format!("Requested address {}{} not a valid socket.", ips[0], args[3]).as_str(), ctx)
                                    }
                                }
                            },
                            Err(_) => {
                                Self::send_message(format!("Could not resolve host of {}!", args[0]).as_str(), ctx)
                            }
                        }
                    } else {
                        Self::send_message("Usage: udp probe 127.0.0.1 40 - Sends a packet to the ip on port 40, returning a success or failure message", ctx)
                    }
                },
                "flood" => {
                    if args.len() >= 4 {
                        match lookup_host(args[2]) {
                            Ok(ips) => {
                                match udp::construct(format!("{}:{}", ips[0], args[3]).to_string()) {
                                    Ok(sckt) => {
                                        sckt.flood(self.config.payload.clone().to_string(), args[4].parse::<u32>().unwrap())
                                    },
                                    Err(_) => ()
                                }
                            },
                            Err(_) => {
                                Self::send_message(format!("Could not resolve host of {}!", args[0]).as_str(), ctx)
                            }
                        }
                    } else {
                        Self::send_message("udp flood 127.0.0.1 40 500 -- Floods a UDP socket on port 40 with 500 packets", ctx)
                    }
                }
                _ => {
                    Self::send_message(format!("udp => {}; {} Unknown Command!", args[1], args[1]).as_str(), ctx)
                }
            }
        } else {
            Self::send_message(format!("Sub-Commands:|nl|  - probe|nl|  - flood").as_str(), ctx)
        }
    }

    pub fn config(&mut self, content: String, ctx: &mut WebsocketContext<CommandHandler>) {
        let args: Vec<&str> = content.split(" ").collect();

        if args.len() == 1 { // Clause if no other parameters are passed
            let mut settings = String::from("Config:|nl|");
            for field in ["payload"].iter() {
                settings.push_str(&format!("  - {}", field.to_string()))
            }
            return Self::send_message(settings.as_str(), ctx)
        }

        if args.len() == 2 || (args[1] != "get" && args[1] != "set") {
            return Self::send_message("Usage: config get/set setting <New Value>|nl|  - config set payload Hello World!", ctx)
        }

        if args.len() >= 3 && args[1] == "get" {
            let value: String = match args[2] {
                "payload" => self.config.payload.clone(),
                _ => String::from("Unknown Field")
            };
            return Self::send_message(value.as_str(), ctx)
        }

        if args.len() > 3 && args[1] == "set" {
            // Trim the input down to just the remainder
            let data = args.clone()
                .drain(3..)
                .as_slice()
                .join(" ");
            self.config.payload = data;
            Self::send_message(format!("Successfully updated field {}", args[2]).as_str(), ctx)
        } else {
            Self::send_message("Usage: config get/set setting <New Value>|nl|  - config set payload Hello World!", ctx)
        }
    }
} 