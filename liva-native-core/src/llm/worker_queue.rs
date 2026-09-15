//! Multi-User Concurrency Queue & Local AI Worker Regulation (R5).
//!
//! Controls and serializes concurrent access to the shared LLM engine (CUDA VRAM),
//! enforcing bounded backpressure (max 32 queued requests) and emitting queue state
//! telemetry (position, estimated wait time) to clients.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const DEFAULT_MAX_CONCURRENCY: usize = 1;
pub const MAX_QUEUE_DEPTH: usize = 32;
pub const ESTIMATED_WAIT_PER_REQUEST_MS: u64 = 1500;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct QueueStatePayload {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_estimate_ms: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct QueueEvent {
    pub event: &'static str,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_estimate_ms: Option<u64>,
    pub payload: QueueStatePayload,
    pub data: QueueStatePayload,
}

impl QueueEvent {
    pub fn queued(position: usize, wait_estimate_ms: u64) -> Self {
        let payload = QueueStatePayload {
            status: "queued".to_string(),
            position: Some(position),
            wait_estimate_ms: Some(wait_estimate_ms),
        };
        Self {
            event: "ai_queue_state",
            status: "queued".to_string(),
            position: Some(position),
            wait_estimate_ms: Some(wait_estimate_ms),
            payload: payload.clone(),
            data: payload,
        }
    }

    pub fn active() -> Self {
        let payload = QueueStatePayload {
            status: "active".to_string(),
            position: None,
            wait_estimate_ms: None,
        };
        Self {
            event: "ai_queue_state",
            status: "active".to_string(),
            position: None,
            wait_estimate_ms: None,
            payload: payload.clone(),
            data: payload,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueueError {
    QueueSaturated { current: usize, max: usize },
    Closed,
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::QueueSaturated { current, max } => {
                write!(
                    f,
                    "AI Worker queue is saturated: {} waiting requests (max {})",
                    current, max
                )
            }
            QueueError::Closed => write!(f, "AI Worker queue semaphore is closed"),
        }
    }
}

impl std::error::Error for QueueError {}

/// RAII Guard that releases the semaphore permit and updates active/processed counters upon drop.
pub struct QueueGuard {
    _permit: OwnedSemaphorePermit,
    active_count: Arc<AtomicUsize>,
    total_processed: Arc<AtomicU64>,
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);
        self.total_processed.fetch_add(1, Ordering::SeqCst);
    }
}

struct WaitingDecrementGuard<'a> {
    counter: &'a AtomicUsize,
    armed: bool,
}

impl<'a> Drop for WaitingDecrementGuard<'a> {
    fn drop(&mut self) {
        if self.armed {
            self.counter.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AiQueueStats {
    pub concurrency: usize,
    pub active: usize,
    pub waiting: usize,
    pub max_queue_depth: usize,
    pub total_processed: u64,
    pub total_rejected: u64,
}

/// Concurrency regulator and bounded request queue for local AI inference.
pub struct AiWorkerQueue {
    semaphore: Arc<Semaphore>,
    concurrency: usize,
    max_queue_depth: usize,
    wait_estimate_ms: u64,
    waiting_count: Arc<AtomicUsize>,
    active_count: Arc<AtomicUsize>,
    total_processed: Arc<AtomicU64>,
    total_rejected: Arc<AtomicU64>,
}

impl AiWorkerQueue {
    pub fn from_env() -> Self {
        let concurrency = std::env::var("LIVA_AI_MAX_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(DEFAULT_MAX_CONCURRENCY);
        Self::new(concurrency)
    }

    pub fn new(concurrency: usize) -> Self {
        Self::with_depth(concurrency, MAX_QUEUE_DEPTH)
    }

    pub fn with_depth(concurrency: usize, max_queue_depth: usize) -> Self {
        let concurrency = concurrency.max(1);
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            concurrency,
            max_queue_depth,
            wait_estimate_ms: ESTIMATED_WAIT_PER_REQUEST_MS,
            waiting_count: Arc::new(AtomicUsize::new(0)),
            active_count: Arc::new(AtomicUsize::new(0)),
            total_processed: Arc::new(AtomicU64::new(0)),
            total_rejected: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn stats(&self) -> AiQueueStats {
        AiQueueStats {
            concurrency: self.concurrency,
            active: self.active_count.load(Ordering::Relaxed),
            waiting: self.waiting_count.load(Ordering::Relaxed),
            max_queue_depth: self.max_queue_depth,
            total_processed: self.total_processed.load(Ordering::Relaxed),
            total_rejected: self.total_rejected.load(Ordering::Relaxed),
        }
    }

    pub async fn acquire(&self) -> Result<QueueGuard, QueueError> {
        self.acquire_with_feedback(|_| {}).await
    }

    pub async fn acquire_with_feedback<F>(&self, mut on_event: F) -> Result<QueueGuard, QueueError>
    where
        F: FnMut(QueueEvent),
    {
        // 1. Fast path: If a permit is immediately available and nobody is waiting, grab it directly
        if self.waiting_count.load(Ordering::SeqCst) == 0
            && let Ok(permit) = self.semaphore.clone().try_acquire_owned()
        {
            self.active_count.fetch_add(1, Ordering::SeqCst);
            on_event(QueueEvent::active());
            return Ok(QueueGuard {
                _permit: permit,
                active_count: Arc::clone(&self.active_count),
                total_processed: Arc::clone(&self.total_processed),
            });
        }

        // 2. Bound check before incrementing
        let cur = self.waiting_count.load(Ordering::SeqCst);
        if cur >= self.max_queue_depth {
            self.total_rejected.fetch_add(1, Ordering::SeqCst);
            return Err(QueueError::QueueSaturated {
                current: cur,
                max: self.max_queue_depth,
            });
        }

        // 3. Atomically increment waiting count
        let pos = self.waiting_count.fetch_add(1, Ordering::SeqCst) + 1;
        if pos > self.max_queue_depth {
            self.waiting_count.fetch_sub(1, Ordering::SeqCst);
            self.total_rejected.fetch_add(1, Ordering::SeqCst);
            return Err(QueueError::QueueSaturated {
                current: pos - 1,
                max: self.max_queue_depth,
            });
        }

        // Emit queued state event
        let wait_ms = (pos as u64) * self.wait_estimate_ms;
        on_event(QueueEvent::queued(pos, wait_ms));

        // RAII guard decrements waiting_count if caller cancels the future
        let mut waiting_guard = WaitingDecrementGuard {
            counter: &self.waiting_count,
            armed: true,
        };

        let permit = match self.semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => return Err(QueueError::Closed),
        };

        // Transition from queued to active
        waiting_guard.armed = false;
        self.waiting_count.fetch_sub(1, Ordering::SeqCst);
        self.active_count.fetch_add(1, Ordering::SeqCst);

        // Emit active state event
        on_event(QueueEvent::active());

        Ok(QueueGuard {
            _permit: permit,
            active_count: Arc::clone(&self.active_count),
            total_processed: Arc::clone(&self.total_processed),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    #[tokio::test]
    async fn test_ai_worker_queue_serialization() {
        let queue = Arc::new(AiWorkerQueue::new(1));
        let concurrent_violation = Arc::new(AtomicBool::new(false));

        // Task 1 acquires and holds permit
        let q1 = Arc::clone(&queue);
        let cv1 = Arc::clone(&concurrent_violation);
        let t1 = tokio::spawn(async move {
            let _guard = q1.acquire().await.expect("task 1 acquire");
            if q1.stats().active > 1 {
                cv1.store(true, Ordering::SeqCst);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            if q1.stats().active > 1 {
                cv1.store(true, Ordering::SeqCst);
            }
        });

        // Give t1 time to acquire
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Task 2 attempts acquire while t1 is holding
        let q2 = Arc::clone(&queue);
        let cv2 = Arc::clone(&concurrent_violation);
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);

        let t2 = tokio::spawn(async move {
            let _guard = q2
                .acquire_with_feedback(|evt| {
                    events_clone.lock().unwrap().push(evt);
                })
                .await
                .expect("task 2 acquire");

            if q2.stats().active > 1 {
                cv2.store(true, Ordering::SeqCst);
            }
        });

        let (r1, r2) = tokio::join!(t1, t2);
        r1.expect("task 1 panicked");
        r2.expect("task 2 panicked");

        assert!(
            !concurrent_violation.load(Ordering::SeqCst),
            "Active count must never exceed 1"
        );

        let recorded = events.lock().unwrap().clone();
        assert_eq!(
            recorded.len(),
            2,
            "Task 2 should have queued then active event"
        );
        assert_eq!(recorded[0].status, "queued");
        assert_eq!(recorded[0].position, Some(1));
        assert_eq!(recorded[1].status, "active");
        assert_eq!(queue.stats().total_processed, 2);
    }

    #[tokio::test]
    async fn test_ai_worker_queue_fifo_positions() {
        let queue = Arc::new(AiWorkerQueue::new(1));

        // Hold initial permit
        let guard_initial = queue.acquire().await.expect("initial permit");
        assert_eq!(queue.stats().active, 1);

        let p1 = Arc::new(std::sync::Mutex::new(0usize));
        let p2 = Arc::new(std::sync::Mutex::new(0usize));
        let p3 = Arc::new(std::sync::Mutex::new(0usize));

        let q1 = Arc::clone(&queue);
        let p1_clone = Arc::clone(&p1);
        let h1 = tokio::spawn(async move {
            let _g = q1
                .acquire_with_feedback(|evt| {
                    if evt.status == "queued" {
                        *p1_clone.lock().unwrap() = evt.position.unwrap_or(0);
                    }
                })
                .await
                .unwrap();
        });
        tokio::time::sleep(Duration::from_millis(15)).await;

        let q2 = Arc::clone(&queue);
        let p2_clone = Arc::clone(&p2);
        let h2 = tokio::spawn(async move {
            let _g = q2
                .acquire_with_feedback(|evt| {
                    if evt.status == "queued" {
                        *p2_clone.lock().unwrap() = evt.position.unwrap_or(0);
                    }
                })
                .await
                .unwrap();
        });
        tokio::time::sleep(Duration::from_millis(15)).await;

        let q3 = Arc::clone(&queue);
        let p3_clone = Arc::clone(&p3);
        let h3 = tokio::spawn(async move {
            let _g = q3
                .acquire_with_feedback(|evt| {
                    if evt.status == "queued" {
                        *p3_clone.lock().unwrap() = evt.position.unwrap_or(0);
                    }
                })
                .await
                .unwrap();
        });
        tokio::time::sleep(Duration::from_millis(15)).await;

        assert_eq!(*p1.lock().unwrap(), 1);
        assert_eq!(*p2.lock().unwrap(), 2);
        assert_eq!(*p3.lock().unwrap(), 3);
        assert_eq!(queue.stats().waiting, 3);

        // Release initial permit and let tasks complete
        drop(guard_initial);

        let (r1, r2, r3) = tokio::join!(h1, h2, h3);
        r1.unwrap();
        r2.unwrap();
        r3.unwrap();

        assert_eq!(queue.stats().waiting, 0);
        assert_eq!(queue.stats().active, 0);
        assert_eq!(queue.stats().total_processed, 4);
    }

    #[tokio::test]
    async fn test_ai_worker_queue_saturation_rejection_bounded() {
        let max_depth = 3;
        let queue = Arc::new(AiWorkerQueue::with_depth(1, max_depth));

        // Hold initial permit
        let guard_initial = queue.acquire().await.expect("initial permit");
        assert_eq!(queue.stats().active, 1);

        // Queue 3 tasks to saturate
        let mut handles = Vec::new();
        for _ in 0..max_depth {
            let q = Arc::clone(&queue);
            handles.push(tokio::spawn(async move {
                let _g = q.acquire().await.unwrap();
            }));
            tokio::time::sleep(Duration::from_millis(15)).await;
        }

        assert_eq!(queue.stats().waiting, 3);

        // 4th waiting request MUST be rejected
        let rejected = queue.acquire().await;
        assert_eq!(
            rejected.err(),
            Some(QueueError::QueueSaturated { current: 3, max: 3 })
        );
        assert_eq!(queue.stats().total_rejected, 1);

        // Release initial permit and finish all queued
        drop(guard_initial);
        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(queue.stats().waiting, 0);
        assert_eq!(queue.stats().active, 0);
    }

    #[tokio::test]
    async fn test_ai_worker_queue_saturation_32_max() {
        let queue = Arc::new(AiWorkerQueue::new(1));
        let guard_initial = queue.acquire().await.expect("initial permit");

        let mut handles = Vec::new();
        for _ in 0..MAX_QUEUE_DEPTH {
            let q = Arc::clone(&queue);
            handles.push(tokio::spawn(async move {
                let _g = q.acquire().await.unwrap();
            }));
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        assert_eq!(queue.stats().waiting, 32);

        // 33rd request exceeds MAX_QUEUE_DEPTH (32) and is rejected
        let res = queue.acquire().await;
        assert_eq!(
            res.err(),
            Some(QueueError::QueueSaturated {
                current: 32,
                max: 32
            })
        );
        assert_eq!(queue.stats().total_rejected, 1);

        drop(guard_initial);
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(queue.stats().waiting, 0);
    }

    #[tokio::test]
    async fn test_ai_worker_queue_cancellation_cleanup() {
        let queue = Arc::new(AiWorkerQueue::new(1));
        let guard_initial = queue.acquire().await.expect("initial permit");

        let q_clone = Arc::clone(&queue);
        let waiter = tokio::spawn(async move {
            let _g = q_clone.acquire().await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(queue.stats().waiting, 1);

        // Abort waiter before it acquires
        waiter.abort();
        let _ = waiter.await;

        // Waiting count should be cleaned up by Drop guard
        assert_eq!(queue.stats().waiting, 0);

        drop(guard_initial);
    }
}
