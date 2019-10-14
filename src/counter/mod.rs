use hashbrown::HashMap;
use std::sync::{Arc, Mutex};
use std::time;

pub type Tags = Arc<Mutex<HashMap<String, Recored>>>;

#[derive(Clone)]
pub struct Counter {
    tags: Tags,
    interval: u64,
}

impl std::fmt::Debug for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("counter")
            .field("interval", &self.interval)
            .finish()
    }
}

pub struct Tracer {
    tags: Tags,
    tag: String,
    interval: u64,

    begin_at: time::SystemTime,
}

pub struct Recored {
    lasttime: time::Duration,
    count: u64,
    min: time::Duration,
    max: time::Duration,
    total: time::Duration,
}

impl Counter {
    pub fn new(interval: u64) -> Counter {
        Counter {
            tags: Arc::new(Mutex::new(HashMap::new())),
            interval,
        }
    }

    pub fn start(&self, tag: String) -> Tracer {
        let tags = self.tags.clone();
        Tracer {
            tags,
            tag,
            interval: self.interval,
            begin_at: time::SystemTime::now(),
        }
    }
}

impl Drop for Tracer {
    fn drop(&mut self) {
        let mut tags = self.tags.lock().unwrap();
        if let Some(mut entry) = tags.get_mut(&self.tag) {
            let now = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap();
            if now < entry.lasttime + time::Duration::from_secs(self.interval) {
                let cost = self.begin_at.elapsed().unwrap();
                if cost > entry.max {
                    entry.max = cost;
                } else if cost < entry.min {
                    entry.min = cost;
                }
                entry.count += 1;
                entry.total += cost;
                return;
            } else {
                log::info!(
                    "{:>30} {:>6} {:>4.3?} {:>4.3?} {:>4.3?}",
                    self.tag,
                    entry.count,
                    entry.min,
                    entry.max,
                    entry.total / entry.count as u32
                );
            }
        }
        let record = Recored {
            lasttime: self.begin_at.duration_since(time::UNIX_EPOCH).unwrap(),
            count: 1,
            min: self.begin_at.elapsed().unwrap(),
            max: self.begin_at.elapsed().unwrap(),
            total: self.begin_at.elapsed().unwrap(),
        };
        tags.insert(self.tag.clone(), record);
    }
}

#[cfg(test)]
mod test {
    use super::Counter;
    #[test]
    fn test_counter() {
        env_logger::from_env(
            env_logger::Env::default()
                .default_filter_or(std::env::var("LOG_LEVEL").unwrap_or(String::from("debug"))),
        )
        .init();
        let counter = Counter::new(2);
        let c = counter.clone();
        let mut handles = vec![];
        let h = std::thread::spawn(move || {
            for i in 0..50 {
                foo(&c, i);
            }
        });
        handles.push(h);
        let c = counter.clone();
        let h = std::thread::spawn(move || {
            for i in 0..50 {
                bar(&c, i);
            }
        });
        handles.push(h);
        for h in handles {
            h.join().unwrap();
        }
    }

    fn foo(counter: &Counter, index: usize) {
        let _tracer = counter.start("foo".to_owned());
        std::thread::sleep(std::time::Duration::from_millis(index as u64 * 10));
    }
    fn bar(counter: &Counter, index: usize) {
        let _tracer = counter.start("bar".to_owned());
        std::thread::sleep(std::time::Duration::from_millis(index as u64 * 20));
    }
}
