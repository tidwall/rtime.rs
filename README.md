# rtime.rs

[![license](https://img.shields.io/crates/l/rtime_rs.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/d/rtime_rs.svg)](https://crates.io/crates/rtime_rs)
[![version](https://img.shields.io/crates/v/rtime_rs.svg?)](https://crates.io/crates/rtime_rs/)
[![documentation](https://docs.rs/rtime_rs/badge.svg?)](https://docs.rs/rtime_rs/)


Retrieve the current time from remote servers.

It works by requesting timestamps from twelve very popular hosts over https.
As soon as it gets at least three responses, it takes the two that have the
smallest difference in time. And from those two it picks the one that is
the oldest. Finally it ensures that the time is monotonic.

## Using

Get the remote time with `rtime_rs::now()`.

```rust
// Get the current internet time, returns chrono::DateTime<Utc>.
// Fails if the Internet is offline.
let now = rtime_rs::now().unwrap();  

println!("{}", now);

// OUTPUT: 
// 2022-05-11 23:05:49 UTC
```

## Stay in sync

The `rtime_rs::now()` will be a little slow, usually 200 ms or more, because it
must make a round trip to three or more remote servers to determine the correct
time. 

You can make it fast like the built-in `std::time::SystemTime::now()` by calling `rtime_rs::sync()` once at the start of your program.

```rust
// Start syncing with the Internet time. Timeouts after 15 seconds when the 
// Internet is offline.
rtime_rs::sync(Duration::from_secs(15)).unwrap(); 

// All following rtime_rs::now() calls will now be quick and without the need
// for checking its result, because they will never fail.
let now = rtime_rs::now().unwrap();  

println!("{}", now);

// OUTPUT:
// 2022-05-11 23:06:52.000072083 UTC
```

It's a good idea to call `rtime_rs::sync()` at the top of the `main()` function.
