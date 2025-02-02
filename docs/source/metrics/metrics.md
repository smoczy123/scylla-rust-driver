# Driver metrics

During operation the driver collects various metrics.

They can be accessed at any moment using `Session::get_metrics()`

### Collected metrics:
* Query latencies
* Total number of nonpaged queries
* Number of errors during nonpaged queries
* Total number of paged queries
* Number of errors during paged queries
* Number of retries

### Example
```rust
# extern crate scylla;
# use scylla::client::session::Session;
# use std::error::Error;
# async fn check_only_compiles(session: &Session) -> Result<(), Box<dyn Error>> {
let metrics = session.get_metrics();

println!("Queries requested: {}", metrics.get_queries_num());
println!("Iter queries requested: {}", metrics.get_queries_iter_num());
println!("Errors occurred: {}", metrics.get_errors_num());
println!("Iter errors occurred: {}", metrics.get_errors_iter_num());
println!("Average latency: {}", metrics.get_latency_avg_ms().unwrap());
println!(
    "99.9 latency percentile: {}",
    metrics.get_latency_percentile_ms(99.9).unwrap()
);
# Ok(())
# }
```