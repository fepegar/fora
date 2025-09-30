use crate::azure::{Experiment, Job, JobDetails};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct CachedData<T> {
    pub data: T,
    pub timestamp: SystemTime,
}

impl<T> CachedData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            timestamp: SystemTime::now(),
        }
    }

    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.timestamp.elapsed().unwrap_or(Duration::MAX) > max_age
    }
}

#[derive(Debug, Clone)]
pub struct CacheManager {
    jobs: Arc<DashMap<String, CachedData<Vec<Job>>>>,
    job_details: Arc<DashMap<String, CachedData<JobDetails>>>,
    experiments: Arc<DashMap<String, CachedData<Vec<Experiment>>>>,
    experiment_jobs: Arc<DashMap<String, CachedData<Vec<Job>>>>,
    default_ttl: Duration,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(DashMap::new()),
            job_details: Arc::new(DashMap::new()),
            experiments: Arc::new(DashMap::new()),
            experiment_jobs: Arc::new(DashMap::new()),
            default_ttl: Duration::from_secs(300), // 5 minutes default TTL
        }
    }

    pub async fn store_jobs(&self, jobs: Vec<Job>) {
        self.jobs
            .insert("recent".to_string(), CachedData::new(jobs));
    }

    pub fn get_jobs(&self) -> Vec<Job> {
        self.jobs
            .get("recent")
            .and_then(|cached| {
                if cached.is_expired(self.default_ttl) {
                    None
                } else {
                    Some(cached.data.clone())
                }
            })
            .unwrap_or_default()
    }

    pub fn get_jobs_cached(&self) -> Vec<Job> {
        // Get jobs regardless of expiration for immediate display
        self.jobs
            .get("recent")
            .map(|cached| cached.data.clone())
            .unwrap_or_default()
    }

    pub async fn store_job_details(&self, job_id: String, details: JobDetails) {
        self.job_details.insert(job_id, CachedData::new(details));
    }

    pub fn get_job_details(&self, job_id: &str) -> Option<JobDetails> {
        self.job_details.get(job_id).and_then(|cached| {
            if cached.is_expired(self.default_ttl) {
                None
            } else {
                Some(cached.data.clone())
            }
        })
    }

    pub fn get_job_details_cached(&self, job_id: &str) -> Option<JobDetails> {
        // Get job details regardless of expiration for immediate display
        self.job_details
            .get(job_id)
            .map(|cached| cached.data.clone())
    }

    pub async fn store_experiments(&self, experiments: Vec<Experiment>) {
        self.experiments
            .insert("all".to_string(), CachedData::new(experiments));
    }

    pub fn get_experiments(&self) -> Vec<Experiment> {
        self.experiments
            .get("all")
            .and_then(|cached| {
                if cached.is_expired(self.default_ttl) {
                    None
                } else {
                    Some(cached.data.clone())
                }
            })
            .unwrap_or_default()
    }

    pub fn get_experiments_cached(&self) -> Vec<Experiment> {
        // Get experiments regardless of expiration for immediate display
        self.experiments
            .get("all")
            .map(|cached| cached.data.clone())
            .unwrap_or_default()
    }

    pub async fn store_experiment_jobs(&self, experiment_id: String, jobs: Vec<Job>) {
        self.experiment_jobs
            .insert(experiment_id, CachedData::new(jobs));
    }

    pub fn get_experiment_jobs(&self, experiment_id: &str) -> Vec<Job> {
        self.experiment_jobs
            .get(experiment_id)
            .and_then(|cached| {
                if cached.is_expired(self.default_ttl) {
                    None
                } else {
                    Some(cached.data.clone())
                }
            })
            .unwrap_or_default()
    }

    pub fn get_experiment_jobs_cached(&self, experiment_id: &str) -> Vec<Job> {
        // Get experiment jobs regardless of expiration for immediate display
        self.experiment_jobs
            .get(experiment_id)
            .map(|cached| cached.data.clone())
            .unwrap_or_default()
    }

    pub fn clear_expired(&self) {
        // Remove expired entries to prevent memory growth
        let ttl = self.default_ttl;

        self.jobs.retain(|_, cached| !cached.is_expired(ttl));
        self.job_details.retain(|_, cached| !cached.is_expired(ttl));
        self.experiments.retain(|_, cached| !cached.is_expired(ttl));
        self.experiment_jobs
            .retain(|_, cached| !cached.is_expired(ttl));
    }
}
