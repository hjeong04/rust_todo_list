use rust_todo_list::ThreadPool;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
#[derive(Serialize, Deserialize, Debug)]
struct Todo {
    id: u32,
    task: String,
    completed: bool,
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

fn add_todo(task: String) {
    let mut todos = load_todos().unwrap();
    let id = todos.len() as u32 + 1;
    let todo = Todo {
        id,
        task,
        completed: false,
    };
    todos.push(todo);
    save_todos(&todos).unwrap();
}

fn list_todos() {
    let todos = load_todos().unwrap();
    for todo in todos {
        println!("{:?}", todo);
    }
}

fn complete_todo(id: u32) {
    let mut todos = load_todos().unwrap();
    if let Some(todo) = todos.iter_mut().find(|todo| todo.id == id) {
        todo.completed = true;
    }
    save_todos(&todos).unwrap();
}

fn delete_todo(id: u32) {
    let mut todos = load_todos().unwrap();
    todos.retain(|todo| todo.id != id);
    save_todos(&todos).unwrap();
}

fn main() {
    // loop {
    //     println!("Select an option:");
    //     println!("1. Add a new task");
    //     println!("2. List all tasks");
    //     println!("3. Mark a task as completed");
    //     println!("4. Delete a task");
    //     println!("5. Exit");

    //     print!("Enter your choice: ");
    //     io::stdout().flush().unwrap(); // Ensure the prompt is displayed

    //     let mut choice = String::new();
    //     io::stdin().read_line(&mut choice).unwrap();

    //     match choice.trim() {
    //         "1" => {
    //             print!("Enter the task description: ");
    //             io::stdout().flush().unwrap();
    //             let mut task = String::new();
    //             io::stdin().read_line(&mut task).unwrap();
    //             add_todo(task.trim().to_string());
    //         }
    //         "2" => list_todos(),
    //         "3" => {
    //             print!("Enter the ID of the task to mark as completed: ");
    //             io::stdout().flush().unwrap();
    //             let mut id = String::new();
    //             io::stdin().read_line(&mut id).unwrap();
    //             if let Ok(id) = id.trim().parse() {
    //                 complete_todo(id);
    //             } else {
    //                 println!("Invalid ID. Please enter a number.");
    //             }
    //         }
    //         "4" => {
    //             print!("Enter the ID of the task to delete: ");
    //             io::stdout().flush().unwrap();
    //             let mut id = String::new();
    //             io::stdin().read_line(&mut id).unwrap();
    //             if let Ok(id) = id.trim().parse() {
    //                 delete_todo(id);
    //             } else {
    //                 println!("Invalid ID. Please enter a number.");
    //             }
    //         }
    //         "5" => {
    //             println!("Exiting...");
    //             break;
    //         }
    //         _ => println!("Invalid choice. Please try again."),
    //     }
    // }
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if (buffer.starts_with(sleep)) {
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
