use std::io::{Read, Write};
use std::path::Path;
use std::{fs::File, time::Duration};

use derivative::Derivative;

use crate::task::{ScheduleTask, Task, TaskRecord};

#[derive(Derivative)]
#[derivative(Debug, Clone, Default)]
pub struct ScheduleConfiguration {
    #[derivative(Default(value = "45"))]
    pub breaktime: u64,
    #[derivative(Default(value = "180"))]
    pub break_frequency: u64,

    #[derivative(Default(value = "8"))]
    pub minibreaktime: u64,
    #[derivative(Default(value = "45"))]
    pub minibreak_frequency: u64,

    #[derivative(Default(value = "3"))]
    pub transitiontime: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ExpectedRatioTasks(pub Vec<(Task, f32)>);

impl ExpectedRatioTasks {
    /// all tasks ratios should add up to `1`
    pub fn new(tasks: Vec<(Task, f32)>) -> Result<Self, ()> {
        let total_ratio = tasks.iter().fold(0.0, |acc, e| acc + e.1);
        let error_range = 0.01;

        if total_ratio > 1.0 + error_range || total_ratio < 1.0 - error_range {
            return Err(());
        }

        Ok(Self(tasks))
    }

    pub fn write<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        let data: Vec<((String, String), f32)> = self
            .0
            .iter()
            .map(|v| ((v.0.name.clone(), v.0.group.clone()), v.1))
            .collect();

        let mut file = File::create(path).unwrap();
        write!(file, "{}", serde_json::to_string_pretty(&data).unwrap()).unwrap();
    }

    pub fn read<P>(path: P, tasks: Vec<Task>) -> Self
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path).unwrap();
        let mut rawdata = String::new();
        file.read_to_string(&mut rawdata).unwrap();

        let data: Vec<((String, String), f32)> = serde_json::from_str(rawdata.as_str()).unwrap();
        let mut output: Vec<(Task, f32)> = Vec::with_capacity(tasks.len());
        let mut used_tasks: Vec<Task> = Vec::with_capacity(tasks.len());

        for task in tasks.iter() {
            let task_ratio_opt = data
                .iter()
                .find(|t| t.0 == (task.name.clone(), task.group.clone()));

            if let Some(task_ratio) = task_ratio_opt {
                used_tasks.push(task.clone());
                output.push((task.clone(), task_ratio.1));
            }
        }

        for task in tasks.iter().filter(|v| {
            used_tasks
                .iter()
                .find(|v2| (v.name.clone(), v.group.clone()) == (v2.name.clone(), v2.group.clone()))
                .is_none()
        }) {
            let index = output.len() - 1;
            let last_f32 = output.get_mut(index).unwrap().1;
            output.push((task.clone(), last_f32 / 2.0));
            let last = output.get_mut(index).unwrap();
            last.1 /= 2.0;
        }

        Self(output)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scheduler {
    pub task_history: Vec<TaskRecord>,
    pub config: ScheduleConfiguration,
    pub tasks: ExpectedRatioTasks,
}

impl Scheduler {
    pub fn new(
        tasks: ExpectedRatioTasks,
        task_history: Vec<TaskRecord>,
        config: ScheduleConfiguration,
    ) -> Self {
        Self {
            tasks,
            config,
            task_history,
            ..Default::default()
        }
    }

    pub fn feed_record(&mut self, record: TaskRecord) {
        self.task_history.push(record);
    }

    fn compute_history_ratio_tasks(
        &self,
        future_factor: Vec<TaskRecord>,
    ) -> Vec<((String, String), f32)> {
        let mut ratio_tasks: Vec<((String, String), f32)> = Vec::new();

        let mut task_history: Vec<TaskRecord> = self
            .task_history
            .clone()
            .iter()
            .filter(|&t| t.origin_group != "system/minibreak" || t.origin_group != "system/break")
            .cloned()
            .collect();

        let mut ff = future_factor.clone();
        task_history.append(&mut ff);

        for record in task_history.iter() {
            let identity = (record.origin_name.clone(), record.origin_group.clone());

            if ratio_tasks.iter().find(|&t| t.0 == identity).is_none() {
                ratio_tasks.push((identity, 0.0));
            }
        }

        for (record, _) in self.tasks.0.iter() {
            let identity = (record.name.clone(), record.group.clone());

            if ratio_tasks.iter().find(|&t| t.0 == identity).is_none() {
                ratio_tasks.push((identity, 0.0));
            }
        }

        let total_time = task_history
            .iter()
            .fold(Duration::from_secs(0), |acc, i| acc + i.time);

        ratio_tasks
            .iter()
            .map(|i| {
                let record = task_history
                    .iter()
                    .filter(|&t| t.origin_name == i.0.0 && t.origin_group == i.0.1)
                    .fold(Duration::default(), |acc, i| acc + i.time);

                if total_time.as_secs() == 0 {
                    return (i.0.clone(), 0.0);
                }

                (
                    i.0.clone(),
                    (record.as_secs_f32() / total_time.as_secs_f32()),
                )
            })
            .collect()
    }

    fn compute_cost(&self, history: Vec<((String, String), f32)>) -> f32 {
        let cost = self.tasks.0.iter().fold(0.0, |acc, a| {
            let identity = (a.0.name.clone(), a.0.group.clone());

            if let Some(record) = history.iter().find(|&r| r.0 == identity) {
                let x = a.1;
                let y = record.1;
                acc + (x - y).abs()
            } else {
                acc + 0.0
            }
        });

        return cost;
    }

    fn compute_breaks(&self, schedule: Vec<ScheduleTask>) -> Vec<ScheduleTask> {
        let mut since_last_minibreak = Duration::from_secs(0);
        let mut since_last_break = Duration::from_secs(0);

        let mut breaked_schedule = vec![];

        for task in schedule {
            if since_last_break >= Duration::from_secs(self.config.break_frequency * 60) {
                since_last_break = Duration::from_secs(0);
                breaked_schedule.push(ScheduleTask {
                    origin_name: String::from("Break"),
                    origin_group: String::from("system/break"),
                    time: Duration::from_secs(self.config.breaktime * 60),
                })
            } else if since_last_minibreak
                >= Duration::from_secs(self.config.minibreak_frequency * 60)
            {
                since_last_minibreak = Duration::from_secs(0);
                breaked_schedule.push(ScheduleTask {
                    origin_name: String::from("Minibreak"),
                    origin_group: String::from("system/minibreak"),
                    time: Duration::from_secs(self.config.minibreaktime * 60),
                })
            }

            breaked_schedule.push(task.clone());

            if task.origin_group == "system/minibreak" {
                since_last_minibreak = Duration::from_secs(0);
            } else {
                since_last_minibreak += task.time.clone();
            }

            if task.origin_group == "system/break" {
                since_last_break = Duration::from_secs(0);
            } else {
                since_last_break += task.time.clone();
            }
        }

        breaked_schedule
    }

    pub fn compute_task(&self, focus_time: Duration, limit: u32) -> Vec<ScheduleTask> {
        let mut virtual_task_history = vec![];
        let mut virtual_history_time = Duration::default();

        for _ in 0..limit {
            if virtual_history_time > focus_time {
                break;
            }

            let mut lowest_task = TaskRecord::default();
            let mut previous_cost = f32::MAX;

            for (task, _) in self.tasks.0.iter() {
                let future_task = TaskRecord {
                    origin_name: task.name.clone(),
                    origin_group: task.group.clone(),
                    time: task.config.time.clone(),
                };

                let mut future_factor = vec![future_task.clone()];
                future_factor.append(&mut virtual_task_history.clone());

                let history = self.compute_history_ratio_tasks(future_factor);
                let cost = self.compute_cost(history);

                // println!(
                //     "{:.3?} {:.3?} {}",
                //     previous_cost, cost, future_task.origin_group
                // );

                if previous_cost > cost {
                    lowest_task = future_task;
                    previous_cost = cost;
                }
            }

            if previous_cost == f32::MAX {
                break;
            }

            virtual_history_time += lowest_task.time.clone();
            // println!("{}", lowest_task.origin_group);
            virtual_task_history.push(lowest_task);

            // let history = self.compute_history_ratio_tasks(virtual_task_history.clone());

            // for record in history.iter() {
            //     println!("{} {}", record.0.0, record.1);
            // }

            // let cost = self.compute_cost(history);
            // println!("{}\n", cost);
            // println!("");
        }

        self.compute_breaks(
            virtual_task_history
                .iter()
                .map(|i| ScheduleTask {
                    origin_name: i.origin_name.clone(),
                    origin_group: i.origin_group.clone(),
                    time: i.time,
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct Schedule {
    pub tasks: Vec<ScheduleTask>,
}
