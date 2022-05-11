#[macro_use]
extern crate lazy_static;

use chrono::{DateTime, NaiveDateTime, Utc};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Info {
    sremote: i64,
    slocal: i64,
    synced: bool,
}

static HOSTS: [&str; 12] = [
    "facebook.com",
    "microsoft.com",
    "amazon.com",
    "google.com",
    "youtube.com",
    "twitter.com",
    "reddit.com",
    "netflix.com",
    "bing.com",
    "twitch.tv",
    "myshopify.com",
    "wikipedia.org",
];

lazy_static! {
    static ref INFO: Mutex<Info> = Mutex::new(Info {
        sremote: 0,
        slocal: 0,
        synced: false,
    });
    static ref RTIME: Mutex<i64> = Mutex::new(0);
    static ref START: Instant = Instant::now();
}

fn elapsed() -> i64 {
    START.elapsed().as_nanos() as i64
}

pub fn now() -> Result<DateTime<Utc>, Box<dyn Error>> {
    let now = {
        let info = INFO.lock().unwrap();
        if info.synced {
            info.sremote + (elapsed() - info.slocal)
        } else {
            drop(info);
            remote_now()?
        }
    };
    let secs = now / 1000000000;
    let nsecs = (now - (secs * 1000000000)) as u32;
    Ok(DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(secs, nsecs),
        Utc,
    ))
}

fn http_get(host: &str) -> Result<i64, Box<dyn Error>> {
    let agent: ureq::Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(2))
        .timeout_write(Duration::from_secs(2))
        .build();
    if let Some(date) = agent
        .head(&format!("http://{}", host))
        .call()?
        .header("date")
    {
        Ok(DateTime::parse_from_rfc2822(date)?.timestamp_nanos())
    } else {
        Err(From::from("missing date header"))
    }
}

fn remote_now() -> Result<i64, Box<dyn Error>> {
    let res = Arc::new(Mutex::new(vec![]));
    for host in HOSTS {
        let res = res.clone();
        std::thread::spawn(move || {
            if let Ok(now) = http_get(host) {
                let mut res = res.lock().unwrap();
                res.push(now);
            }
        });
    }
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        let res = res.lock().unwrap();
        if res.len() < 3 {
            drop(res);
            std::thread::sleep(Duration::from_millis(20));
            continue;
        }
        let mut list = Vec::new();
        for i in 0..res.len() {
            for j in i + 1..res.len() {
                if i != j {
                    let (mut tm0, mut tm1) = (res[i], res[j]);
                    if tm0 > tm1 {
                        (tm0, tm1) = (tm1, tm0)
                    }
                    list.push((tm0, tm1, tm1 - tm0))
                }
            }
        }
        list.sort_by(|a, b| a.2.cmp(&b.2));
        let res = list[0].0;
        // Ensure that the new time is after the previous time.
        let mut rtime = RTIME.lock().unwrap();
        if res > *rtime {
            *rtime = res;
        }
        return Ok(*rtime);
    }
    Err(From::from("Internet offline"))
}

pub fn sync(timeout: Duration) -> Result<(), Box<dyn Error>> {
    let mut info = INFO.lock().unwrap();
    if info.synced {
        return Ok(());
    }
    let mut now = 0;
    let start = Instant::now();
    while now == 0 && start.elapsed() < timeout {
        if let Ok(tm) = remote_now() {
            now = tm;
        }
    }
    if now == 0 {
        return Err(From::from("Internet offline"));
    }
    info.synced = true;
    info.sremote = now;
    info.slocal = elapsed();
    drop(info);

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(15));
        let now = match remote_now() {
            Ok(now) => now,
            Err(_) => continue,
        };
        let mut info = INFO.lock().unwrap();
        if now > info.sremote {
            info.sremote = now;
            info.slocal = elapsed();
        }
    });
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_now() {
        sync(Duration::from_secs(15)).unwrap();
        // let now = now().unwrap();
        // println!("{}", now);

        assert!(now().unwrap() <= now().unwrap());
        assert!(now().unwrap() <= now().unwrap());
        assert!(now().unwrap() <= now().unwrap());
    }
}
