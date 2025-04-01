use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use serde_json::Result;
use std::env;

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
    let todo = Todo { id, task, completed: false };
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

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "add" => add_todo(args[2].clone()),
        "list" => list_todos(),
        "complete" => complete_todo(args[2].parse().unwrap()),
        _ => println!("Unknown command"),
    }
}
