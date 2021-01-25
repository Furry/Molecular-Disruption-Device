use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, rt};
use actix_web_actors::ws;

use super::command_handler;

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(command_handler::command_handler_new(), &req, stream);
    println!("{:?}", resp);
    resp
}

async fn root() -> String {
    println!("HIT ROOT");
    String::from("OK")
}

pub fn ws_init() {
    let handler = command_handler::command_handler_new();
    // let mutx = web::Data::new(Mutex::new(handler));
    let mut sys = rt::System::new("listener");
    let srv = HttpServer::new(move || {
        App::new()
        .app_data(handler.clone())
        .route("/ws/", web::get().to(index))
        .route("/", web::get().to(root))
    })
    .bind("127.0.0.1:8080").unwrap()
    .run();
    sys.block_on(srv).unwrap();
}