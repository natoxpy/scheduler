use crate::task::{Task, TaskRecord};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub struct FileStorage<P>
where
    P: AsRef<Path>,
{
    pub tasks: P,
    pub records: P,
    pub tasks_ratios: P,
}

impl<P> FileStorage<P>
where
    P: AsRef<Path>,
{
    pub fn new(tasks: P, records: P, tasks_ratios: P) -> Self {
        Self {
            tasks,
            records,
            tasks_ratios,
        }
    }
}

impl Storable<Task> for FileStorage<&'static str> {
    // FIX: unwraps
    fn store(&self, data: &Vec<Task>) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.tasks)
            .unwrap();

        let data = serde_json::to_string_pretty(&data).unwrap();
        write!(file, "{}", data).unwrap();
    }

    // FIX: unwraps
    fn get(&self) -> Vec<Task> {
        let path = Path::new(self.tasks);
        if !path.exists() {
            File::create(&path).unwrap();
        }

        let mut file = File::open(&path).unwrap();

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        serde_json::from_str(&buf).unwrap()
    }
}

impl Storable<TaskRecord> for FileStorage<&'static str> {
    // FIX: unwraps
    fn store(&self, data: &Vec<TaskRecord>) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.records)
            .unwrap();

        let data = serde_json::to_string_pretty(&data).unwrap();
        write!(file, "{}", data).unwrap();
    }

    // FIX: unwraps
    fn get(&self) -> Vec<TaskRecord> {
        let path = Path::new(self.records);
        if !path.exists() {
            File::create(&path).unwrap();
        }

        let mut file = File::open(&path).unwrap();

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        serde_json::from_str(&buf).unwrap()
    }
}

impl Storable<((String, String), f32)> for FileStorage<&'static str> {
    fn store(&self, data: &Vec<((String, String), f32)>) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.tasks_ratios)
            .unwrap();

        let data = serde_json::to_string_pretty(&data).unwrap();
        write!(file, "{}", data).unwrap();
    }

    fn get(&self) -> Vec<((String, String), f32)> {
        let path = Path::new(self.tasks_ratios);
        if !path.exists() {
            File::create(&path).unwrap();
        }

        let mut file = File::open(&path).unwrap();

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        serde_json::from_str(&buf).unwrap()
    }
}

pub trait Storable<T> {
    fn store(&self, data: &Vec<T>);
    fn get(&self) -> Vec<T>;
}
