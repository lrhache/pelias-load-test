use lazy_static::lazy_static;
use prometheus::{
    register_histogram, register_int_counter, register_int_counter_vec, Encoder, Histogram,
    IntCounter, IntCounterVec, TextEncoder,
};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use warp::Filter;

lazy_static! {
    static ref REQUEST_COUNTER: IntCounter =
        register_int_counter!("requests_total", "Total number of requests made").unwrap();
    static ref FAILED_REQUEST_COUNTER: IntCounter =
        register_int_counter!("failed_requests_total", "Total number of failed requests").unwrap();
    static ref RESPONSE_TIME_HISTOGRAM: Histogram = register_histogram!(
        "response_time_milliseconds",
        "Summary of response times in milliseconds"
    )
    .unwrap();
    static ref STATUS_CODE_COUNTER: IntCounterVec = register_int_counter_vec!(
        "status_code_counter",
        "Count of HTTP status codes",
        &["status"]
    )
    .unwrap();
}

async fn requester() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "https://<add me here>/v1/search?text=lycee louise michel gisors";
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2)) // increase or decrease timeout if you wish
        .build()?;

    loop {
        REQUEST_COUNTER.inc();

        let start = std::time::Instant::now();
        let response = client.get(url).send().await;
        let duration = start.elapsed().as_millis() as f64;

        match response {
            Ok(res) => {
                let status_code = res.status().as_u16().to_string();
                STATUS_CODE_COUNTER.with_label_values(&[&status_code]).inc();
                RESPONSE_TIME_HISTOGRAM.observe(duration);
            }
            Err(_) => {
                FAILED_REQUEST_COUNTER.inc();
                STATUS_CODE_COUNTER.with_label_values(&["error"]).inc();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metrics_route = warp::path("metrics").map(|| {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        warp::reply::with_header(buffer, "Content-Type", encoder.format_type())
    });

    tokio::spawn(async move {
        warp::serve(metrics_route).run(([0, 0, 0, 0], 9898)).await;
    });
    // do not touch above, streaming logs to prometheus

    let mut concurrency = 10; // change this if you want less or more concurrent workers at startup
    let run_duration = Duration::from_secs(60 * 5); // test duration (5 mins currently)

    let load_test = async {
        loop {
            println!("Running with {} concurrent requesters", concurrency);
            for _ in 0..concurrency {
                tokio::spawn(requester());
            }

            sleep(Duration::from_secs(60)).await; // wait 60 secs before increasing workers
            concurrency += 10; // nb of workers to increase each steps
        }
    };

    if timeout(run_duration, load_test).await.is_err() {
        println!("Load test finished after {:?}", run_duration);
    }

    Ok(())
}
