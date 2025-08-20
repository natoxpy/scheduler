use crate::storage::Storable;
use crate::task::{ScheduleTask, Task, TaskRecord};
use derivative::Derivative;
use std::time::Duration;

#[derive(Derivative)]
#[derivative(Debug, Clone, Default)]
pub struct ScheduleConfiguration {
    #[derivative(Default(value = "45"))]
    pub breaktime: u64,
    #[derivative(Default(value = "180"))]
    pub break_frequency: u64,

    #[derivative(Default(value = "10"))]
    pub minibreaktime: u64,
    #[derivative(Default(value = "45"))]
    pub minibreak_frequency: u64,

    #[derivative(Default(value = "3"))]
    pub transitiontime: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ExpectedRatioTasks(pub Vec<(Task, f32)>);

impl ExpectedRatioTasks {
    pub fn new(tasks: Vec<(Task, f32)>) -> Result<Self, ()> {
        let total_ratio = tasks.iter().fold(0.0, |acc, e| acc + e.1);
        let error_range = 0.01;

        if total_ratio > 1.0 + error_range || total_ratio < 1.0 - error_range {
            return Err(());
        }

        Ok(Self(tasks))
    }

    pub fn write<P>(&self, storage: &P)
    where
        P: Storable<((String, String), f32)>,
    {
        let data: Vec<((String, String), f32)> = self
            .0
            .iter()
            .map(|v| ((v.0.name.clone(), v.0.group.clone()), v.1))
            .collect();

        storage.store(&data);
    }

    pub fn read<P>(storage: &P, tasks: Vec<Task>) -> Self
    where
        P: Storable<((String, String), f32)>,
    {
        let data: Vec<((String, String), f32)> = storage.get();

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

        Self::new(output).unwrap()
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

    fn compute_break(&self, virtual_history: &Vec<TaskRecord>) -> Option<ScheduleTask> {
        let mut total_history = self.task_history.clone();
        total_history.append(&mut virtual_history.clone());

        let mut since_last_minibreak = Duration::from_secs(0);
        let mut since_last_break = Duration::from_secs(0);

        for record in total_history {
            since_last_minibreak += record.time;
            since_last_break += record.time;

            if record.origin_group == "system/break" {
                since_last_break = Duration::from_secs(0);
                since_last_minibreak = Duration::from_secs(0);
            }

            if record.origin_group == "system/minibreak" {
                since_last_minibreak = Duration::from_secs(0);
            }
        }

        println!("{:?}", since_last_break);

        if since_last_break >= Duration::from_secs(self.config.break_frequency * 60) {
            Some(ScheduleTask {
                origin_name: String::from("Break"),
                origin_group: String::from("system/break"),
                time: Duration::from_secs(self.config.breaktime * 60),
            })
        } else if since_last_minibreak >= Duration::from_secs(self.config.minibreak_frequency * 60)
        {
            Some(ScheduleTask {
                origin_name: String::from("Minibreak"),
                origin_group: String::from("system/minibreak"),
                time: Duration::from_secs(self.config.minibreaktime * 60),
            })
        } else {
            None
        }
    }

    pub fn compute_task(&self, virtual_history: &Vec<TaskRecord>) -> ScheduleTask {
        let break_task = self.compute_break(&virtual_history);

        if let Some(break_schedule) = break_task {
            return break_schedule;
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
            future_factor.append(&mut virtual_history.clone());

            let history = self.compute_history_ratio_tasks(future_factor);
            let cost = self.compute_cost(history);

            if previous_cost > cost {
                lowest_task = future_task;
                previous_cost = cost;
            }
        }

        ScheduleTask {
            origin_name: lowest_task.origin_name.clone(),
            origin_group: lowest_task.origin_group.clone(),
            time: lowest_task.time,
        }
    }

    pub fn compute_tasks(
        &self,
        virtual_history: &Vec<TaskRecord>,
        limit: usize,
    ) -> Vec<ScheduleTask> {
        let mut future_schedule = Vec::with_capacity(limit);

        for _ in 0..limit {
            let mut temp_virtual_history = Vec::with_capacity(virtual_history.len());
            temp_virtual_history.append(&mut virtual_history.clone());

            let mut future_history = future_schedule
                .iter()
                .cloned()
                .map(|v| TaskRecord::from(v))
                .collect();

            temp_virtual_history.append(&mut future_history);

            future_schedule.push(self.compute_task(&temp_virtual_history));
        }

        future_schedule
    }
}
