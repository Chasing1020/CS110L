mod request;
mod response;

use clap::Parser;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
// use std::sync::{mpsc::channel, Arc};
// use threadpool::ThreadPool;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::delay_for;

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[clap(about = "Fun with load balancing")]
struct CmdOptions {
    #[clap(
        short,
        long,
        help = "IP/port to bind to",
        default_value = "0.0.0.0:8080"
    )]
    bind: String,
    #[clap(short, long, help = "Upstream host to forward requests to")]
    upstream: Vec<String>,
    #[clap(
        long,
        help = "Perform active health checks on this interval (in seconds)",
        default_value = "10"
    )]
    active_health_check_interval: usize,
    #[clap(
        long,
        help = "Path to send request to for active health checks",
        default_value = "/"
    )]
    active_health_check_path: String,
    #[clap(
        long,
        help = "Maximum number of requests to accept per IP per minute (0 = unlimited)",
        default_value = "0"
    )]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    #[allow(dead_code)]
    active_health_check_interval: usize,
    /// Where we should send requests when doing active health checks (Milestone 4)
    #[allow(dead_code)]
    active_health_check_path: String,
    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    #[allow(dead_code)]
    max_requests_per_minute: usize,
    /// Addresses of servers that we are proxying to
    upstream_addresses: Vec<String>,
    /// Upstream status, id -> true/false
    available_addresses: RwLock<HashSet<usize>>,
    /// request_count
    request_count: Mutex<HashMap<IpAddr, usize>>,
}

#[tokio::main]
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // Start listening for connections
    let mut listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    let available_addresses: HashSet<usize> = (0..options.upstream.len()).collect();
    let request_count = Mutex::new(HashMap::new());
    // Handle incoming connections
    let state = ProxyState {
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        available_addresses: RwLock::new(available_addresses),
        request_count,
    };

    // // finish Milestone 1: Add multithreading
    // let n_workers = 8;
    // let pool = ThreadPool::new(n_workers);
    // let shared_state = Arc::new(state);
    // for stream in listener.incoming() {
    //     let state_copy = shared_state.clone();
    //     if let Ok(stream) = stream {
    //         // Handle the connection!
    //         pool.execute(move || handle_connection(stream, &state_copy));
    //     }
    // }

    let shared_state = Arc::new(state);

    let health_check_state = shared_state.clone();
    tokio::spawn(async move {
        start_health_check(&health_check_state).await;
    });

    if shared_state.max_requests_per_minute > 0 {
        let request_count_state = shared_state.clone();
        tokio::spawn(async move {
            refresh_request_count(&request_count_state).await;
        });
    }

    // finish Milestone 2: Add async
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((mut stream, _)) => {
                let ip_addr = stream.peer_addr().unwrap().ip();
                let mut request_count = shared_state.request_count.lock().await;
                let count = request_count.entry(ip_addr).or_insert(0);
                *count += 1;
                println!(
                    "***count:{}, max:{}***",
                    count, shared_state.max_requests_per_minute
                );
                let max_requests_per_minute = shared_state.max_requests_per_minute;
                if max_requests_per_minute == 0 || *count <= max_requests_per_minute {
                    let shared_state_ref = Arc::clone(&shared_state);
                    tokio::spawn(async move {
                        handle_connection(stream, &shared_state_ref).await;
                    });
                } else {
                    let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
                    response::write_to_stream(&response, &mut stream)
                        .await
                        .unwrap();
                    continue;
                }
            }
            Err(_) => {
                break;
            }
        }
    }
    log::error!("Connection closed");
}

async fn start_health_check(state: &ProxyState) {
    loop {
        delay_for(Duration::from_secs(
            state.active_health_check_interval as u64,
        ))
        .await;
        let mut available_addresses_writer = state.available_addresses.write().await;
        for idx in 0..state.upstream_addresses.len() {
            let upstream_ip = &state.upstream_addresses[idx];
            let uri = &state.active_health_check_path;
            if check_http_status(uri, upstream_ip).await {
                // if !TcpStream::connect(upstream_ip).await.is_err() {
                available_addresses_writer.insert(idx);
            } else {
                available_addresses_writer.remove(&idx);
            }
        }
    }
}

async fn check_http_status(uri: &String, upstream_ip: &String) -> bool {
    let mut stream = TcpStream::connect(upstream_ip).await.unwrap();
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(uri)
        .header("Host", upstream_ip)
        .body(Vec::new())
        .unwrap();
    let _ = request::write_to_stream(&request, &mut stream).await.ok();
    let res = response::read_from_stream(&mut stream, &http::Method::GET)
        .await
        .ok()
        .unwrap();
    res.status().as_u16() == 200
}

async fn refresh_request_count(state: &ProxyState) {
    loop {
        delay_for(Duration::from_secs(60)).await;
        let mut request_count = state.request_count.lock().await;
        request_count.clear();
    }
}

async fn pickup_random_alive_upstream(state: &ProxyState) -> Option<usize> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    // get the read lock, release automatically when the function return
    let available_addresses_reader = state.available_addresses.read().await;
    if available_addresses_reader.is_empty() {
        return None;
    }
    let random_idx = rng.gen_range(0, available_addresses_reader.len());
    if let Some(&index) = available_addresses_reader.iter().nth(random_idx) {
        return Some(index);
    }
    None
}

async fn connect_to_upstream(state: &ProxyState) -> Result<TcpStream, std::io::Error> {
    // let mut rng = rand::rngs::StdRng::from_entropy();
    loop {
        if let Some(upstream_idx) = pickup_random_alive_upstream(state).await {
            eprintln!("++++status: {:?} ++++", state.available_addresses.read().await);
            let upstream_ip = &state.upstream_addresses[upstream_idx];
            match TcpStream::connect(upstream_ip).await {
                Ok(stream) => return Ok(stream),
                Err(_) => {
                    let mut available_addresses_writer = state.available_addresses.write().await;
                    available_addresses_writer.remove(&upstream_idx);
                }
            };
        } else {
            return Err(std::io::Error::new(
                ErrorKind::Unsupported,
                "All the upstream servers are down!",
            ));
        }
    }
}

async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!(
        "{} <- {}",
        client_ip,
        response::format_response_line(&response)
    );
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

async fn handle_connection(mut client_conn: TcpStream, state: &ProxyState) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
    };
    let upstream_ip = client_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!(
                "Failed to send request to upstream {}: {}",
                upstream_ip,
                error
            );
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await
        {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
