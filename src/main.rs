extern crate scheduler;

use scheduler::schedule::{ExpectedRatioTasks, ScheduleConfiguration, Scheduler};
use scheduler::storage::{FileStorage, Storable};
use scheduler::task::{ScheduleTask, Task, TaskRecord};
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

fn clear_screen() {
    // Clear entire screen, move cursor to top-left
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn main() {
    let storage = FileStorage::new("../tasks.json", "../history.json", "../task-ratio.json");

    let tasks: Vec<Task> = storage.get();
    let history: Vec<TaskRecord> = storage.get();
    let ratioed_tasks = ExpectedRatioTasks::read(&storage, tasks);

    let mut scheduler = Scheduler::new(ratioed_tasks, history, ScheduleConfiguration::default());

    let mut schedule_tasks = scheduler.compute_tasks(&vec![], 15);

    let origin_time = Instant::now();
    let mut current_task = schedule_tasks.remove(0);
    let mut since_last_start = Instant::now();

    loop {
        clear_screen();

        if current_task.time.as_secs() - since_last_start.elapsed().as_secs() == 0 {
            if current_task.origin_group != "system/transition" {
                since_last_start = Instant::now();
                scheduler.feed_record(TaskRecord::from(current_task));

                storage.store(&scheduler.task_history);

                current_task = ScheduleTask {
                    origin_name: String::from("Transition"),
                    origin_group: String::from("system/transition"),
                    time: Duration::from_secs(60 * 3),
                };
            } else {
                since_last_start = Instant::now();
                current_task = schedule_tasks.remove(0);
            }
        }

        println!("------");
        println!(
            "{} - {} {}",
            format_duration(origin_time.elapsed()),
            format_duration(current_task.time - since_last_start.elapsed()),
            current_task.origin_name
        );
        println!("------");

        for task in schedule_tasks.iter() {
            println!("({:2?}m) {}", task.time.as_secs() / 60, task.origin_group);
        }

        thread::sleep(Duration::new(1, 0));
    }
}
