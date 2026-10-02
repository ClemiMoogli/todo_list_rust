use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use tauri::{State, AppHandle, Manager};
use std::fs;


#[derive(Clone, Serialize, Deserialize)]
enum TaskState{
    NotStarted,
    Finished,
}

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    name: String,
    state: TaskState,
    id: u32,
}

struct Data {
    tasks: Mutex<Vec<Task>>,
    next_id: AtomicU32,
}


fn save_tasks(app: &AppHandle, tasks: &Vec<Task>) {
    let json = serde_json::to_string_pretty(tasks).unwrap();
    println!("{}", json);
    let dir = app.path().app_data_dir().unwrap();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("task.json");
    fs::write(&path, json).unwrap();
    println!("Sauvegarde dans: {:?}", path);
}

fn load_tasks(app: &AppHandle) -> Vec<Task> {
    let path = app.path().app_data_dir().unwrap().join("task.json");
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap(),
        Err(_) => Vec::new(),
    }
}


#[tauri::command]
fn create_task(task_name: String, app: AppHandle, data: State<Data>) -> Result<Vec<Task>, String> {
    if task_name != "" {
        let task = Task {
            name: task_name,
            state: TaskState::NotStarted,
            id: data.next_id.fetch_add(1, Ordering::Relaxed),
        };
        let mut tasks = data.tasks.lock().unwrap();
        tasks.push(task);
        save_tasks(&app, &tasks);
        Ok(tasks.clone())
    } else {
        Err(String::from("Empty task_name."))
    }
}

#[tauri::command]
fn delete_task(task_id: u32, app: AppHandle, data: State<Data>) -> Vec<Task> {
    let mut tasks = data.tasks.lock().unwrap();
    tasks.retain(|t| t.id != task_id);
    save_tasks(&app, &tasks);
    tasks.clone()
}

#[tauri::command]
fn validate_task(task_id: u32, app: AppHandle, data: State<Data>) -> Result<Vec<Task>, String> {
    let mut tasks = data.tasks.lock().unwrap();
    let task = tasks.iter_mut().find(|t| t.id == task_id);
    match task {
        Some(task) => {
            task.state = match task.state {
                TaskState::NotStarted => TaskState::Finished,
                TaskState::Finished => TaskState::NotStarted,
            };
        },
        None => return Err(String::from("Invalid task id during the validation.")),
    }
    save_tasks(&app, &tasks);
    Ok(tasks.clone())
}

#[tauri::command]
fn show_task(data: State<Data>) -> Vec<Task> {
    let tasks = data.tasks.lock().unwrap();
    tasks.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let tasks = load_tasks(app.handle());
            let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            app.manage(Data {
                tasks : Mutex::new(tasks),
                next_id : AtomicU32::new(next_id),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![create_task, delete_task, validate_task, show_task])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
