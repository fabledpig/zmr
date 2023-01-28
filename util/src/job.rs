use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

pub struct JobHandle<T> {
    result: Arc<Mutex<Option<T>>>,
    condvar: Arc<Condvar>,
}

impl<T> JobHandle<T> {
    pub fn new(result: Arc<Mutex<Option<T>>>, condvar: Arc<Condvar>) -> Self {
        Self { result, condvar }
    }

    pub fn wait(self) -> T {
        let mut result = self.result.lock().unwrap();
        while result.is_none() {
            result = self.condvar.wait(result).unwrap();
        }

        result.take().unwrap()
    }
}

struct Job {
    fun: Box<dyn FnOnce() + Send>,
}

impl Job {
    fn new<T: Send + 'static, U: FnOnce() -> T + Send + 'static>(fun: U) -> (Self, JobHandle<T>) {
        let result = Arc::new(Mutex::new(None));
        let condvar = Arc::new(Condvar::new());
        let job_handle = JobHandle::new(result.clone(), condvar.clone());

        let fun = move || {
            let ret = fun();
            *result.lock().unwrap() = Some(ret);
            condvar.notify_all();
        };

        (Self { fun: Box::new(fun) }, job_handle)
    }

    fn execute(self) {
        (self.fun)();
    }
}

pub trait ThreadCategory: Copy + Eq + Hash {}

#[derive(Eq, Hash, PartialEq)]
pub struct ThreadCategoryDescriptor<T: ThreadCategory> {
    thread_category: T,
    number_of_threads: usize,
}

impl<T: ThreadCategory> ThreadCategoryDescriptor<T> {
    pub fn new(thread_category: T, number_of_threads: usize) -> Self {
        Self {
            thread_category,
            number_of_threads,
        }
    }
}

pub trait ThreadPoolDescriptor<T: ThreadCategory> {
    fn thread_category_descriptors(&self) -> HashSet<ThreadCategoryDescriptor<T>>;
}

#[macro_export]
macro_rules! thread_pool {
    ($i:ident, $($j:ident:$k:literal),*) => {
        pub mod $i {
            use super::smart_enum;
            use super::ThreadCategoryDescriptor;
            use std::collections::HashSet;

            smart_enum!(pub, ThreadCategory, $($j),*);

            impl super::ThreadCategory for ThreadCategory {}

            pub struct ThreadPoolDescriptor {}

            impl super::ThreadPoolDescriptor<ThreadCategory> for ThreadPoolDescriptor {
                fn thread_category_descriptors(&self) -> HashSet<ThreadCategoryDescriptor<ThreadCategory>> {
                    let mut thread_category_descriptors = HashSet::new();
                    $(thread_category_descriptors.insert(ThreadCategoryDescriptor::new(ThreadCategory::$j, $k));)*
                    thread_category_descriptors
                }
            }
        }
    };
}

struct JobQueueState {
    jobs: VecDeque<Job>,
    should_stop: bool,
}

impl JobQueueState {
    fn new() -> Self {
        Self {
            jobs: VecDeque::new(),
            should_stop: false,
        }
    }
}

pub struct Scheduler<T> {
    job_queue_states: HashMap<T, Arc<Mutex<JobQueueState>>>,
    condvars: HashMap<T, Arc<Condvar>>,
    join_handles: Vec<JoinHandle<()>>,
}

impl<T: ThreadCategory> Scheduler<T> {
    pub fn new<U: ThreadPoolDescriptor<T>>(thread_pool_descriptor: U) -> Self {
        let mut job_queue_states = HashMap::new();
        let mut join_handles = Vec::new();
        let mut condvars = HashMap::new();

        for thread_category_descriptor in thread_pool_descriptor.thread_category_descriptors() {
            let job_queue_state = Arc::new(Mutex::new(JobQueueState::new()));
            job_queue_states.insert(
                thread_category_descriptor.thread_category,
                job_queue_state.clone(),
            );

            let condvar = Arc::new(Condvar::new());
            condvars.insert(thread_category_descriptor.thread_category, condvar.clone());

            for _ in 0..thread_category_descriptor.number_of_threads {
                let job_queue_state_mutex = job_queue_state.clone();
                let condvar = condvar.clone();

                let join_handle = thread::spawn(move || {
                    let mut job_queue_state = job_queue_state_mutex.lock().unwrap();

                    loop {
                        if let Some(job) = job_queue_state.jobs.pop_front() {
                            drop(job_queue_state);
                            job.execute();
                            job_queue_state = job_queue_state_mutex.lock().unwrap();
                        } else if job_queue_state.should_stop {
                            break;
                        } else {
                            job_queue_state = condvar.wait(job_queue_state).unwrap();
                        }
                    }
                });
                join_handles.push(join_handle);
            }
        }

        Self {
            job_queue_states,
            condvars,
            join_handles,
        }
    }

    pub fn schedule_job<U: Send + 'static, V: FnOnce() -> U + Send + 'static>(
        &self,
        thread_category: T,
        fun: V,
    ) -> JobHandle<U> {
        let job_handle = {
            let (job, job_handle) = Job::new(fun);
            let mut job_queue_state = self
                .job_queue_states
                .get(&thread_category)
                .unwrap()
                .lock()
                .unwrap();

            job_queue_state.jobs.push_back(job);

            job_handle
        };

        self.condvars.get(&thread_category).unwrap().notify_one();

        job_handle
    }
}

impl<T> Drop for Scheduler<T> {
    fn drop(&mut self) {
        for (_, job_queue_state) in &self.job_queue_states {
            let mut job_queue_state = job_queue_state.lock().unwrap();
            job_queue_state.should_stop = true;
        }

        for (_, condvar) in &self.condvars {
            condvar.notify_all();
        }

        while let Some(join_handle) = self.join_handles.pop() {
            join_handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Scheduler, ThreadCategory, ThreadCategoryDescriptor, ThreadPoolDescriptor};
    use crate::smart_enum;

    thread_pool!(test_thread_pool, Category1: 3, Category2: 3, Category3: 3);

    #[test]
    fn test_scheduler() {
        let scheduler = Scheduler::new(test_thread_pool::ThreadPoolDescriptor {});

        let job_handle_1 = scheduler
            .schedule_job(test_thread_pool::ThreadCategory::Category1, || {
                "#1 Hello from a scheduler thread."
            });

        let job_handle_2 = scheduler
            .schedule_job(test_thread_pool::ThreadCategory::Category2, || {
                "#2 Hello from a scheduler thread."
            });

        let job_handle_3 = scheduler
            .schedule_job(test_thread_pool::ThreadCategory::Category3, || {
                "#3 Hello from a scheduler thread."
            });

        assert_eq!(job_handle_1.wait(), "#1 Hello from a scheduler thread.");
        assert_eq!(job_handle_2.wait(), "#2 Hello from a scheduler thread.");
        assert_eq!(job_handle_3.wait(), "#3 Hello from a scheduler thread.");
    }
}
