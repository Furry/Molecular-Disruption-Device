use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use dns_lookup::lookup_host;

use serde_json;

#[derive(Debug, Clone, Copy)]
pub struct CommandHandler {}

impl Actor for CommandHandler {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CommandHandler {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(Self::call(&self, text)),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl CommandHandler {
    fn call(&self, content: String) -> String {
        let args: Vec<String> = content.split(" ")
            .map(|x| x.to_string())
            .collect();
        let data = args.clone()
            .drain(1..)
            .as_slice()
            .join("");
        println!("{}", args[0]);
        let func: fn(content: String) -> std::string::String = match args[0].as_str() {
            "ping" => Self::ping,
            "resolve" => Self::resolve,
            "echo" => Self::echo,
            _ => Self::default
        };
        func(data)
    }
    
    pub fn default(_content: String) -> String {
        String::from(r#"[{"error": "Command is not known!"}]"#)
    }

    pub fn ping(_content: String) -> String {
        String::from("PONG")
    }

    /// Resolves an input string
    pub fn resolve(content: String) -> String {
        match lookup_host(content.as_str()) {
            Ok(ips) => {
                let mut string_ips: Vec<String> = Vec::new();
                for ip in ips {
                    string_ips.push(ip.to_string());
                };
                String::from(format!("[{{\"message_json\": {}}}]", serde_json::json!(string_ips).to_string()))
            },
            Err(_) => {
                String::from(r#"[{"error": "Null Resolution"}]"#)
            }
        }
    }

    // Echos content
    pub fn echo(content: String) -> String {
        String::from(format!("[{{\"message\": {}}}]", content))
    }

}