use actix_cors::Cors;
use actix_web::{web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct Todo {
    id: u32,
    task: String,
    completed: bool,
}

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
}

struct AppState {
    tasks: Mutex<Vec<Task>>,
}

fn load_todos() -> Result<Vec<Todo>> {
    let file = File::open("todos.json").unwrap_or_else(|_| File::create("todos.json").unwrap());
    let reader = BufReader::new(file);
    let todos: Vec<Todo> = serde_json::from_reader(reader).unwrap_or_else(|_| vec![]);
    Ok(todos)
}

fn save_todos(todos: &Vec<Todo>) -> Result<()> {
    let file = File::create("todos.json").map_err(serde_json::Error::io)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, todos)?;
    Ok(())
}

#[actix_web::post("/add_todo")]
async fn add_todo(data: web::Data<AppState>, task: String) -> impl Responder {
    let mut todos = data.tasks.lock().unwrap();
    let id = todos.len() + 1;
    todos.push(Task {
        id,
        description: task,
        completed: false,
    });
    println!("Task added with ID: {}", id);
    format!("Task added with ID: {}", id)
}

#[actix_web::get("/list_todos")]
async fn list_todos(data: web::Data<AppState>) -> impl Responder {
    let todos = data.tasks.lock().unwrap();
    let todo_list: Vec<String> = todos
        .iter()
        .map(|todo| {
            format!(
                "ID: {}, Task: {}, Completed: {}",
                todo.id, todo.description, todo.completed
            )
        })
        .collect();
    println!("Listing all tasks:");
    println!("{:?}", todo_list);
    todo_list.join("\n")
}

#[actix_web::post("/complete_todo/{id}")]
async fn complete_todo(data: web::Data<AppState>, id: web::Json<usize>) -> impl Responder {
    let mut todos = data.tasks.lock().unwrap();
    if let Some(todo) = todos.iter_mut().find(|todo| todo.id == *id) {
        todo.completed = true;
        format!("Task with ID: {} marked as completed", id)
    } else {
        format!("Task with ID: {} not found", id)
    }
}

#[actix_web::delete("/delete_todo/{id}")]
async fn delete_todo(data: web::Data<AppState>, task_id: web::Path<usize>) -> impl Responder {
    let mut todos = data.tasks.lock().unwrap();
    if let Some(pos) = todos.iter().position(|todo| todo.id == *task_id) {
        todos.remove(pos);
        format!("Task with ID: {} deleted", task_id)
    } else {
        format!("Task with ID: {} not found", task_id)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(AppState {
        tasks: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default() // Enable CORS
                    .allow_any_origin() // Allow requests from any origin
                    .allow_any_method() // Allow any HTTP method
                    .allow_any_header(),
            ) // Allow any header
            .app_data(data.clone())
            .service(add_todo)
            .service(list_todos)
            .service(complete_todo)
            .service(delete_todo)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await

    // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // let pool = ThreadPool::new(4);

    // for stream in listener.incoming().take(2) {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }
    // println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
