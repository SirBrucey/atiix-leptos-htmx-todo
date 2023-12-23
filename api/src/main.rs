use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, RwLock,
};

use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct ApplicationState {
    id: AtomicU32,
    todos: RwLock<Vec<Todo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Todo {
    id: u32,
    title: Arc<str>,
    completed: bool,
}

impl Todo {
    pub fn new(id: u32, title: String) -> Self {
        Self {
            id,
            title: title.into(),
            completed: false,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct AddTodo {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
    let init_state = web::Data::new(ApplicationState {
        id: AtomicU32::new(0),
        todos: RwLock::new(Vec::new()),
    });
    HttpServer::new(move || {
        App::new()
            .service(todos)
            .service(get_todo)
            .service(add_todo)
            .service(update_todo)
            .service(delete_todo)
            .app_data(init_state.clone())
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}

#[get("/")]
async fn todos(state: web::Data<ApplicationState>) -> impl Responder {
    let todos = (*state.todos.read().unwrap()).clone();
    web::Json(todos)
}

#[get("/{id}")]
async fn get_todo(state: web::Data<ApplicationState>, id: web::Path<u32>) -> impl Responder {
    let other_todos: Vec<Todo> = (*state.todos.read().unwrap()).clone();
    if let Some(todo) = other_todos.iter().find(|t| t.id == *id) {
        HttpResponse::Ok().json(todo)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/")]
async fn add_todo(state: web::Data<ApplicationState>, todo: web::Json<AddTodo>) -> impl Responder {
    let mut t = state.todos.write().unwrap();
    t.push(Todo::new(
        state.id.fetch_add(1, Ordering::Relaxed) + 1,
        todo.0.title,
    ));
    HttpResponse::Created()
}

#[put("/{id}")]
async fn update_todo(
    state: web::Data<ApplicationState>,
    id: web::Path<u32>,
    data: web::Json<UpdateTodo>,
) -> impl Responder {
    let mut t = state.todos.write().unwrap();
    match t.iter_mut().find(|t| t.id == *id) {
        Some(todo) => {
            if let Some(title) = &data.title {
                todo.title = Arc::from(title.as_str());
            }
            if let Some(completed) = &data.completed {
                todo.completed = *completed;
            }
            HttpResponse::Ok()
        }
        None => HttpResponse::NotFound(),
    }
}

#[delete("/{id}")]
async fn delete_todo(state: web::Data<ApplicationState>, id: web::Path<u32>) -> impl Responder {
    let mut t = state.todos.write().unwrap();
    match t.iter_mut().position(|t| t.id == *id) {
        Some(todo) => {
            t.remove(todo);
            HttpResponse::NoContent()
        }
        None => HttpResponse::NotFound(),
    }
}
