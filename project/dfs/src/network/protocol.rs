use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::cas::{object::Object, resource::Resource};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectRequest {
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceRequest {
    pub hash: Object,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceResponse {
    pub resource: Resource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedirectResponse {
    pub hash: Object,
    pub node: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailabilityCheckRequest {
    pub hashes: Vec<Object>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailabilityCheckResponse {
    pub hashes: Vec<Object>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Connect(ConnectRequest),
    Resource(ResourceRequest),
    AvailabilityCheck(AvailabilityCheckRequest),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Resource(ResourceResponse),
    Redirect(RedirectResponse),
    AvailabilityCheck(AvailabilityCheckResponse),
    Error,
}
