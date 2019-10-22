use hyper::{client::HttpConnector, Body, Client, Request, Response};
use rayon::prelude::ParallelIterator;
use rayon::prelude::ParallelSlice;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn main() {
    let data = std::fs::read("./output.txt").expect("failed to load file list");
    let client = Client::builder()
        .max_idle_per_host(100)
        .keep_alive(true)
        .build_http();
    // let (counter, total_length) = (std::sync::Arc::new(std::sync::))
    // let counter = Arc::new(AtomicUsize::new(0));
    let counter = Arc::new(AtomicUsize::new(0));
    let total_length = AtomicUsize::new(0);
    let real_speed = Arc::new(AtomicUsize::new(0));
    let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
    {
        let counter = counter.clone();
        let real_speed: Arc<AtomicUsize> = real_speed.clone();
        std::thread::spawn(move || {
            let mut t = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                t += 1;
                let count = counter.load(Ordering::SeqCst);
                let speed = real_speed.swap(0, Ordering::SeqCst);
                println!(
                    "total: {} qps: {:.3?}, speed: {:.3?}",
                    count,
                    count as f64 / t as f64,
                    speed
                );
            }
        });
    }
    data.par_split(|&num| num == '\n' as u8).for_each(|line| {
        let line: &[u8] = &line;
        let url = String::from_utf8(line.to_vec()).unwrap();
        let len = runtime.block_on(get(client.clone(), url)).unwrap().len();
        if len == 0 {
            println!(
                "len is zero. counter: {}, total length: {}",
                counter.load(Ordering::SeqCst),
                total_length.load(Ordering::SeqCst)
            );
        } else {
            counter.fetch_add(1, Ordering::SeqCst);
            total_length.fetch_add(len, Ordering::SeqCst);
            real_speed.fetch_add(1, Ordering::SeqCst);
            // println!(
            //     "counter: {}, total length: {}",
            //     counter.load(Ordering::SeqCst),
            //     total_length.load(Ordering::SeqCst)
            // );
        }
    });
    println!(
        "counter: {}, total length: {}",
        counter.load(Ordering::SeqCst),
        total_length.load(Ordering::SeqCst)
    );
}

async fn get(client: Client<HttpConnector, Body>, url: String) -> Result<Vec<u8>, ()> {
    if url.is_empty() {
        return Ok(vec![]);
    }
    let u = String::from("http://172.21.20.250:8888/server") + &url;
    // let raw_u = u.clone();
    let u: hyper::Uri = u.clone().parse().unwrap();
    let req = Request::get(u).body(Body::empty()).unwrap();
    let response: Response<Body> = match client.request(req).await {
        Ok(resp) => resp,
        Err(err) => {
            println!("failed to get {}, error: {}", url, err);
            return Ok(vec![]);
        }
    };
    let mut body: Body = response.into_body();
    let mut data = vec![];
    while let Some(chunk) = body.next().await {
        let chunk = chunk.unwrap();
        let chunk: &[u8] = &chunk;
        data.extend_from_slice(chunk);
    }
    Ok(data)
}
