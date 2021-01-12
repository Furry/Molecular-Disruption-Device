use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, rt};
use actix_web_actors::ws;

use super::command_handler::CommandHandler;

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(CommandHandler {}, &req, stream);
    println!("{:?}", resp);
    resp
}

async fn root() -> String {
    println!("HIT ROOT");
    String::from("OK")
}

pub fn ws_init() {
    let handler = CommandHandler {};
    let x = CommandHandler::resolve(String::from("mothers.against.vodka"));
    println!("{}", x);
    // let mutx = web::Data::new(Mutex::new(handler));
    let mut sys = rt::System::new("listener");
    let srv = HttpServer::new(move || {
        App::new()
        .app_data(handler)
        .route("/ws/", web::get().to(index))
        .route("/", web::get().to(root))
    })
    .bind("127.0.0.1:8080").unwrap()
    .run();
    sys.block_on(srv).unwrap();
}