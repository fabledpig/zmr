use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::thread::{self};

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
macro_rules! thread_category {
    ($i:ident, $($j:ident),*) => {
        mod thread_category {
            use $crate::smart_enum;
            use $crate::job::ThreadCategory;

            smart_enum!(pub, $i, $($j),*);

            impl ThreadCategory for $i {}
        }

        pub use thread_category::$i;
    };
}

#[macro_export]
macro_rules! thread_pool_descriptor {
    ($i:ident, $($j:ident:$k:literal),*) => {
        mod thread_pool_descriptor {
            use std::collections::HashSet;
            use $crate::job::ThreadPoolDescriptor as ThreadPoolDescriptorTrait;
            use $crate::job::ThreadCategoryDescriptor;
            use super::$i;

            pub struct ThreadPoolDescriptor {}

            impl ThreadPoolDescriptorTrait<$i> for ThreadPoolDescriptor {
                fn thread_category_descriptors(&self) -> HashSet<ThreadCategoryDescriptor<$i>> {
                    let mut thread_category_descriptors = HashSet::new();
                    $(thread_category_descriptors.insert(ThreadCategoryDescriptor::new($i::$j, $k));)*
                    thread_category_descriptors
                }
            }
        }

        pub use thread_pool_descriptor::ThreadPoolDescriptor;
    };
}

#[macro_export]
macro_rules! thread_pool {
    ($i:ident, $($j:ident:$k:literal),*) => {
        mod thread_pool {
            use $crate::thread_category;
            use $crate::thread_pool_descriptor;

            thread_category!($i, $($j),*);
            thread_pool_descriptor!($i, $($j:$k),*);
        }

        pub use thread_pool::$i;
        pub use thread_pool::ThreadPoolDescriptor;
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

    pub fn scoped<'sched, 'env, U>(&'sched self, fun: U)
    where
        U: FnOnce(&ScopedScheduler<'sched, 'env, T>),
    {
        let scoped_job_handles = Arc::new(Mutex::new(Vec::new()));
        let scoped_scheduler = ScopedScheduler::new(self, scoped_job_handles);
        fun(&scoped_scheduler);
    }
}

impl<T> Drop for Scheduler<T> {
    fn drop(&mut self) {
        for job_queue_state in self.job_queue_states.values() {
            let mut job_queue_state = job_queue_state.lock().unwrap();
            job_queue_state.should_stop = true;
        }

        for condvar in self.condvars.values() {
            condvar.notify_all();
        }

        while let Some(join_handle) = self.join_handles.pop() {
            join_handle.join().unwrap();
        }
    }
}

pub struct ScopedJobHandle {
    job_handle: Option<JobHandle<()>>,
}

impl ScopedJobHandle {
    pub fn new(job_handle: JobHandle<()>) -> Self {
        Self {
            job_handle: Some(job_handle),
        }
    }
}

impl Drop for ScopedJobHandle {
    fn drop(&mut self) {
        self.job_handle.take().unwrap().wait();
    }
}

pub struct ScopedScheduler<'sched, 'env, T> {
    scheduler: &'sched Scheduler<T>,
    scoped_job_handles: Arc<Mutex<Vec<ScopedJobHandle>>>,
    env: PhantomData<&'env mut &'env ()>,
}

impl<'sched, 'env, T: ThreadCategory> ScopedScheduler<'sched, 'env, T> {
    fn new(
        scheduler: &'sched Scheduler<T>,
        scoped_job_handles: Arc<Mutex<Vec<ScopedJobHandle>>>,
    ) -> Self {
        Self {
            scheduler,
            scoped_job_handles,
            env: PhantomData,
        }
    }

    pub fn schedule_job<U: FnOnce() + Send + 'env>(&self, thread_category: T, fun: U) {
        let fun: Box<dyn FnOnce() + Send + 'static> = {
            let fun: Box<dyn FnOnce() + Send + 'env> = Box::new(move || {
                fun();
            });
            unsafe { std::mem::transmute(fun) }
        };

        let job_handle = self.scheduler.schedule_job(thread_category, fun);
        self.scoped_job_handles
            .lock()
            .unwrap()
            .push(ScopedJobHandle::new(job_handle));
    }
}

#[cfg(test)]
mod tests {
    use super::Scheduler;

    thread_pool!(TestThreadCategory, Category1: 3, Category2: 3, Category3: 3);

    #[test]
    fn test_scheduler() {
        let scheduler = Scheduler::new(ThreadPoolDescriptor {});

        let job_handle_1 = scheduler.schedule_job(TestThreadCategory::Category1, || {
            "#1 Hello from a scheduler thread."
        });

        let job_handle_2 = scheduler.schedule_job(TestThreadCategory::Category2, || {
            "#2 Hello from a scheduler thread."
        });

        let job_handle_3 = scheduler.schedule_job(TestThreadCategory::Category3, || {
            "#3 Hello from a scheduler thread."
        });

        assert_eq!(job_handle_1.wait(), "#1 Hello from a scheduler thread.");
        assert_eq!(job_handle_2.wait(), "#2 Hello from a scheduler thread.");
        assert_eq!(job_handle_3.wait(), "#3 Hello from a scheduler thread.");
    }

    #[test]
    fn test_scoped_scheduler() {
        let scheduler = Scheduler::new(ThreadPoolDescriptor {});

        let mut s1 = String::new();
        let mut s2 = String::new();
        let mut s3 = String::new();

        scheduler.scoped(|s| {
            s.schedule_job(TestThreadCategory::Category1, || {
                s1 = String::from("s1");
            });
            s.schedule_job(TestThreadCategory::Category2, || {
                s2 = String::from("s2");
            });
            s.schedule_job(TestThreadCategory::Category3, || {
                s3 = String::from("s3");
            });
        });

        assert_eq!(s1, "s1");
        assert_eq!(s2, "s2");
        assert_eq!(s3, "s3");
    }
}
