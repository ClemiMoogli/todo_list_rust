import { invoke } from "@tauri-apps/api/core";

let tasks: Task[] = [];
let listEl: HTMLUListElement;
let errorEl: HTMLParagraphElement;

type TaskState = "NotStarted" | "Finished";

type Task = {
  name: string;
  state: TaskState;
  id: number;
};

async function createTask(name: string) {
    try {
        errorEl.textContent = "";
        tasks = await invoke<Task[]>("create_task", {taskName: name});
        render();
    } catch (err) {
        errorEl.textContent = String(err);
    }
}

async function deleteTask(id: number) {
    tasks = await invoke<Task[]>("delete_task", {taskId: id});
    render();
}

async function validateTask(id: number) {
    try {
        tasks = await invoke<Task[]>("validate_task", {taskId: id});
        errorEl.textContent = "";
        render();
    } catch (err) {
        errorEl.textContent = String(err);
    }
}

async function showTask(){
    tasks = await invoke<Task[]>("show_task");
    render();
}

function render() {
    listEl.replaceChildren();
    for (const task of tasks) {
        const li = document.createElement("li");
        if (task.state === "Finished") {
          li.classList.add("done");
        }
        
        const checkbox = document.createElement("input");
        checkbox.type = "checkbox";
        checkbox.checked = task.state === "Finished";
        checkbox.addEventListener("change", () => validateTask(task.id));

        const span = document.createElement("span");
        span.textContent = task.name;

        const deleteBtn = document.createElement("button");
        deleteBtn.textContent = "X";
        deleteBtn.addEventListener("click", () => deleteTask(task.id));

        li.append(checkbox, span, deleteBtn);
        listEl.append(li);
    }
}

window.addEventListener("DOMContentLoaded", () => {
  const formEl = document.querySelector<HTMLFormElement>("#form-new-task");
  const inputEl = document.querySelector<HTMLInputElement>("#task-input");
  errorEl = document.querySelector("#error")!;
  listEl = document.querySelector<HTMLUListElement>("#task-list")!;
  showTask();
  formEl?.addEventListener("submit", (e) => {
    e.preventDefault();
    if (inputEl) {
      createTask(inputEl.value);
      inputEl.value = "";
    }
  });
});
