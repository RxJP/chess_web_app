use crate::models::{LoginForm, RegistrationForm, SessionDB, User, UserDB};
use crate::stockfish_interface::{StockfishInstance, MAX_CONCURRENT_STOCKFISH_INSTANCES};
use crate::utils::generate_hash;
use chess::{BoardStatus, ChessMove, MoveGen, Piece, Square, EMPTY};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::{Redirect, Responder};
use rocket::serde::json::Json;
use rocket::{Request, State};
use rocket_dyn_templates::{context, Template};
use rocket_ws::{Channel, WebSocket};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// macro_rules! authorization_check {
//     ($cookies: ident, $session_db: ident) => {
//         match $cookies.get_private("session_id") {
//             None => {return display_message("Unauthorised!");}
//             Some(cookie) => {
//                 let sess_id = cookie.value().to_string();
//                 if !$session_db.contains_session(&sess_id) {
//                     $cookies.remove_private("session_id");
//                     return display_message("Unauthorised! Re-Login");
//                 }
//             }
//         }
//     };
// }

#[derive(Debug, Responder)]
pub enum CustomResponse {
    Template(Template),
    Redirect(Redirect),
}

#[get("/")]
pub fn index(cookies: &CookieJar<'_>, session_db: &State<SessionDB>, user_db: &State<UserDB>) -> CustomResponse {
    if let Some(session_id) = cookies.get_private("session_id") {
        match session_db.get(&session_id.value().to_string()) {
            None => {
                cookies.remove_private("session_id");
                return CustomResponse::Redirect(Redirect::to(uri!(auth)));
            }
            Some(email_id) => {
                let user = user_db.get_user_from_email(&email_id).expect("User Not Found!");
                return CustomResponse::Template(Template::render("index", context! {
                    is_authenticated: "true",
                    rating: format!("{}", user.rating),
                    username: user.username
                }));
            }
        }
    }
    else {
        return CustomResponse::Template(Template::render("index", context! {
            is_authenticated: "false",
            rating: "NA",
            username: "Login"
        }));
    }
}

#[get("/auth")]
pub fn auth() -> Template {
    Template::render("auth", context! {})
}

#[derive(Debug, Serialize)]
pub enum AuthResponse {
    ErrorMessage(&'static str),
    RedirectURL(&'static str),
}

#[post("/login", data = "<login_form>")]
pub fn login(login_form: Form<LoginForm>, cookies: &CookieJar<'_>, session_db: &State<SessionDB>, user_db: &State<UserDB>) -> Json<AuthResponse> {
    //todo add a cookies check to check if the user is already logged in
    let user = match &*login_form.login_type {
        "Username" => {
            if !user_db.contains_username(&login_form.id) {
                return Json(AuthResponse::ErrorMessage("Username does not exist!"));
            }
            user_db.get_user_from_username(&login_form.id).unwrap()
        }
        "Email" => {
            if !user_db.contains_email(&login_form.id) {
                return Json(AuthResponse::ErrorMessage("Email does not exist!"));
            }
            user_db.get_user_from_email(&login_form.id).unwrap()
        }
        _ => { return Json(AuthResponse::ErrorMessage("Invalid Request!")); }
    };

    if user.password_hash != generate_hash(login_form.password.as_bytes()) {
        return Json(AuthResponse::ErrorMessage("Wrong password!"));
    }

    loop {
        let new_session_id = generate_hash(&std::time::UNIX_EPOCH.elapsed().expect("Error reading time").as_nanos().to_be_bytes());//potential security vulnerability
        if !session_db.contains_session(&new_session_id) {
            cookies.add_private(Cookie::new("session_id", new_session_id.clone()));
            session_db.insert(new_session_id, user.email);
            break;
        }
        eprintln!("new_session_id: {} already exists in session_db", new_session_id);
    }

    return Json(AuthResponse::RedirectURL("/"));
}

#[post("/register", data = "<registration_form>")]
pub fn register(registration_form: Form<RegistrationForm>, user_db: &State<UserDB>) -> Json<AuthResponse> {
    if user_db.contains_email(&registration_form.email_id) {
        return Json(AuthResponse::ErrorMessage("Email already registered!"));
    } else if user_db.contains_username(&registration_form.username) {
        return Json(AuthResponse::ErrorMessage("Username already exists!"));
    }

    user_db.insert_user(registration_form.email_id.clone(), User {
        username: registration_form.username.clone(),
        email: registration_form.email_id.clone(),
        password_hash: generate_hash(registration_form.password.as_bytes()),
        rating: 500,
    });

    return Json(AuthResponse::RedirectURL("/auth"));
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>, session_db: &State<SessionDB>) -> Status {
    if let Some(session_id) = cookies.get_private("session_id") {
        let _ = session_db.remove_session(&session_id.value().to_string());
        cookies.remove_private("session_id");
    }

    Status::Ok
}

#[get("/is_username_taken/<username>")]
pub fn is_username_taken(username: &str, user_db: &State<UserDB>) -> String {
    user_db.contains_username(&username.to_string()).to_string()
}

#[get("/bot_play")]
pub fn bot_play(cookies: &CookieJar<'_>, session_db: &State<SessionDB>, user_db: &State<UserDB>) -> Template {
    match cookies.get_private("session_id") {
        Some(session_id_cookie) => {
            let session_id = session_id_cookie.value().to_string();
            match session_db.get(&session_id) {
                Some(email_id) => {
                    match user_db.get_user_from_email(&email_id) {
                        Some(user) => {
                            return Template::render("game", context! {
                                is_authenticated: "true",
                                rating: user.rating,
                                username: user.username,
                                opp_username: "Bot",
                                is_bot_play: "true",
                            });
                        }
                        None => {
                            let _ = session_db.remove_session(&session_id);
                        }
                    }
                }
                None => {
                    cookies.remove_private("session_id");
                    return Template::render("index", context! {
                        is_authenticated: "false",
                        rating: "NA",
                        username: "Login"
                    });
                }
            }
        }
        None => {}
    }

    return Template::render("game", context! {
        is_authenticated: "false",
        rating: "NA",
        username: "Guest",
        opp_username: "Bot",
        is_bot_play: "true",
    });
}

#[derive(Debug, Serialize, Deserialize)]
enum JSMessage {
    Connected,
    MovePlayed(JSMove),
    KeepAlive,
}
#[derive(Debug, Serialize, Deserialize)]
struct JSMove {
    from: [u8; 2],
    to: [u8; 2],
    promotion_piece: JSPiece,
}
#[derive(Debug, Serialize, Deserialize)]
enum JSPiece {
    None,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
impl From<Piece> for JSPiece {
    fn from(value: Piece) -> Self {
        match value {
            Piece::Pawn => {JSPiece::Pawn}
            Piece::Knight => {JSPiece::Knight}
            Piece::Bishop => {JSPiece::Bishop}
            Piece::Rook => {JSPiece::Rook}
            Piece::Queen => {JSPiece::Queen}
            Piece::King => {JSPiece::King}
        }
    }
}
impl From<ChessMove> for JSMove {
    fn from(value: ChessMove) -> Self {
        let source_square = value.get_source();
        let destination_square = value.get_dest();
        let promotion_piece =
            if let Some(piece) = value.get_promotion() {
                JSPiece::from(piece)
            }
            else {
                JSPiece::None
            };
        JSMove {
            from: [7 - source_square.get_rank().to_index() as u8, source_square.get_file().to_index() as u8],
            to: [7 - destination_square.get_rank().to_index() as u8, destination_square.get_file().to_index() as u8],
            promotion_piece,
        }
    }
}
fn piece_from_js_piece(value: &JSPiece) -> Option<Piece> {
    Some(match value {
        JSPiece::None => {return None;}
        JSPiece::Pawn => {Piece::Pawn}
        JSPiece::Knight => {Piece::Knight}
        JSPiece::Bishop => {Piece::Bishop}
        JSPiece::Rook => {Piece::Rook}
        JSPiece::Queen => {Piece::Queen}
        JSPiece::King => {Piece::King}
    })
}
fn chess_move_from_js_move(value: &JSMove) -> Option<ChessMove> {
    if value.from[0] > 7 || value.from[1] > 7 || value.to[0] > 7 || value.to[1] > 7 { return None; }
    let from = unsafe { Square::new((7 - value.from[0]) * 8 + value.from[1]) };
    let to = unsafe { Square::new((7 - value.to[0]) * 8 + value.to[1]) };
    let piece = piece_from_js_piece(&value.promotion_piece);
    Some(ChessMove::new(from, to, piece))
}
fn get_all_legal_moves(board: &chess::Board) -> Vec<JSMove> {
    let mut move_iterator = MoveGen::new_legal(&board);
    move_iterator.set_iterator_mask(!EMPTY);
    let mut moves = Vec::with_capacity(move_iterator.len());
    for m in &mut move_iterator {
        moves.push(JSMove::from(m));
    }

    moves
}

#[get("/ws_bot_play")]
pub fn ws_bot_play(ws: WebSocket, stockfish_instance_count: &State<Arc<AtomicU8>>) -> Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};
    use chess::Board;

    let stockfish_instance_count_clone = stockfish_instance_count.inner().clone();
    // println!("===>{:?}", serde_json::to_string(&JSMessage::Connected));

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let stockfish_instance_count = stockfish_instance_count_clone;
            let update_result = stockfish_instance_count.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                if val < MAX_CONCURRENT_STOCKFISH_INSTANCES
                { Some(val + 1) } else { None }
            });
            if update_result.is_err() {
                stream.send("{error: \"Queue Full\"}".into()).await.expect("Error while writing to the ws stream");
                stream.close(None).await.expect("Error while closing the ws stream");
                return Ok(());
            }

            let mut board = Board::default();
            let mut stockfish_instance =
                if let Some(inst) = StockfishInstance::new(stockfish_instance_count).await { inst } else {
                    return Ok(()); //todo handle error
                };

            while let Ok(Some(message)) = timeout(Duration::from_secs(35), stream.next()).await {
                match message {
                    Ok(msg) => {
                        if msg.is_close() { return Ok(()); }
                        let js_message = serde_json::from_str::<JSMessage>(&*msg.to_string());
                        println!("============>{:?}", js_message);
                        if js_message.is_err() {
                            stream.send("{\"error\": \"Invalid JSON\"}".into()).await.expect("Error Occurred while writing to the ws stream");
                            stream.close(None).await.expect("Error Occurred while closing the ws stream");
                            break;
                        }
                        match js_message.unwrap() {
                            JSMessage::Connected => {
                                let moves = get_all_legal_moves(&board);
                                let msg = serde_json::to_string(&moves).expect("Error occurred while serializing moves array");
                                stream.send(msg.into()).await.expect("Error occurred while sending message0!");
                            }
                            JSMessage::MovePlayed(jsmv) => {
                                match chess_move_from_js_move(&jsmv) {
                                    Some(mv) => {
                                        if board.legal(mv) {
                                            stream.send(format!(r#" {{"is_valid": true, "move": {}}} "#, serde_json::to_string(&jsmv).unwrap()).into()).await.expect("Error occurred while sending message1!");
                                            board = board.make_move_new(mv);
                                            match board.status() {
                                                BoardStatus::Ongoing => {}
                                                BoardStatus::Stalemate => { stream.send(r#"{"game_status":"stalemate"}"#.into()).await.expect("Error sending message"); break;}
                                                BoardStatus::Checkmate => { stream.send(r#"{"game_status":"win"}"#.into()).await.expect("Error sending message"); break;}
                                            }
                                            let chess_move = stockfish_instance.get_next_move(mv).await;
                                            let js_move = JSMove::from(chess_move);
                                            board = board.make_move_new(chess_move);
                                            let moves = get_all_legal_moves(&board);
                                            let mut obj = serde_json::Map::new();
                                            obj.insert("opp_move".to_string(), serde_json::to_value(js_move).unwrap());
                                            obj.insert("legal_moves".to_string(), serde_json::to_value(moves).unwrap());
                                            stream.send(serde_json::to_string(&obj).unwrap().into()).await.expect("Error occurred while sending message2.5");

                                            match board.status() {
                                                BoardStatus::Ongoing => {}
                                                BoardStatus::Stalemate => {stream.send(r#"{"game_status":"stalemate"}"#.into()).await.expect("Error sending message"); break;}
                                                BoardStatus::Checkmate => {stream.send(r#"{"game_status":"lose"}"#.into()).await.expect("Error sending message"); break;}
                                            }
                                        } else {
                                            stream.send("{\"is_valid\": false}".into()).await.expect("Error occurred while sending message2!");
                                        }
                                    }
                                    None => {
                                        stream.send("{\"is_valid\": false}".into()).await.expect("Error occurred while sending message3!");
                                    }
                                }
                            }
                            JSMessage::KeepAlive => {}
                        }
                    }
                    Err(err) => {
                        eprintln!("{:?}", err);
                        break;
                    }
                }
            }

            Ok(())
        })
    })
}

pub fn display_message(msg: &str) -> Template {
    Template::render("message", context! {
        title: "Message",
        message: msg
    })
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}
#[catch(500)]
pub fn internal_error(req: &Request<'_>) -> Template {
    Template::render("error/500", context! {
        uri: req.uri()
    })
}

#[cfg(debug_assertions)]
#[get("/debug_internal_state")]
pub fn debug_internal_state(cookies: &CookieJar<'_>, session_db: &State<SessionDB>, user_db: &State<UserDB>) -> String {
    // authorization_check!(cookies, session_db);
    format!("Cookies: {:?}\nSessionDB:\n{:?}\n\nUserDB:\n{:?}", cookies, session_db, user_db)
}

/*
//Add a custom template at runtime
pub fn customize(tera: &mut Tera) {
    tera.add_raw_template("about.html", r#"
        {% extends "base" %}

        {% block content %}
            <section id="about">
              <h1>About - Here's another page!</h1>
            </section>
        {% endblock content %}
    "#).expect("valid Tera template");
}
*/