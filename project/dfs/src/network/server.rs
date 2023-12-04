use std::{
    io::Error,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::{
    cas::{resource::Resource, ContentAddressedStore},
    network::{
        connection::send_packet,
        protocol::{AvailabilityCheckResponse, ResourceResponse, Response},
    },
    store::fs::LocalStore,
};

use super::{connection::recv_packet, fs::NetworkClient, protocol::Request};

pub fn run_server<A: ToSocketAddrs>(
    fs: Arc<Mutex<LocalStore>>,
    client: Arc<Mutex<NetworkClient>>,
    addr: A,
) -> Result<(), Error> {
    let listener = TcpListener::bind(addr)?;
    let incoming = listener.incoming();

    for stream in incoming {
        let stream = stream?;
        let fs = fs.clone();
        let client = client.clone();

        println!("new connection");

        spawn(move || host_connection_loop(fs, client, stream));
    }

    Ok(())
}

fn host_connection_loop(
    fs: Arc<Mutex<LocalStore>>,
    client: Arc<Mutex<NetworkClient>>,
    mut stream: TcpStream,
) {
    loop {
        let request = recv_packet::<Request>(&mut stream);
        match request {
            Ok(Request::Connect(req)) => {
                // todo: remove unwrap
                client.lock().unwrap().add_peer(req.addr).unwrap();
            }
            Ok(Request::Resource(res)) => {
                let fs = fs.lock().unwrap();
                let resource: Option<Resource> = fs.get::<Resource>(&res.hash);
                let response = if let Some(resource) = resource {
                    Response::Resource(ResourceResponse { resource })
                } else {
                    Response::Error
                };
                drop(fs);
                send_packet(&mut stream, &response).unwrap();
            }
            Ok(Request::AvailabilityCheck(ac)) => {
                let fs = fs.lock().unwrap();
                let found_hashes = ac
                    .hashes
                    .iter()
                    .filter(|&hash| fs.has(hash))
                    .cloned()
                    .collect();
                let response = Response::AvailabilityCheck(AvailabilityCheckResponse {
                    hashes: found_hashes,
                });
                drop(fs);
                send_packet(&mut stream, &response).unwrap();
            }
            Err(err) => {
                eprintln!("Got error {} in server", err);
                break;
            }
        }
    }
}

// todo: get rid of 'static somehow
pub fn spawn_server<A: ToSocketAddrs + Send + 'static>(
    fs: Arc<Mutex<LocalStore>>,
    client: Arc<Mutex<NetworkClient>>,
    addr: A,
) {
    spawn(move || run_server(fs, client, addr));
}
