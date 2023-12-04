use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Error, ErrorKind},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

use libc::EINVAL;

use crate::cas::{
    error::PathResolutionError, object::Object, resource::Resource, ContentAddressedStore,
};

use super::{
    connection::{recv_packet, send_packet},
    protocol::{AvailabilityCheckRequest, ConnectRequest, Request, ResourceRequest, Response},
};

pub struct NetworkClient {
    host_address: SocketAddr,
    peers: RefCell<HashMap<SocketAddr, TcpStream>>,
}

impl NetworkClient {
    pub fn new(local_address: SocketAddr) -> Self {
        Self {
            host_address: local_address,
            peers: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_peer<A: ToSocketAddrs>(&mut self, addr: A) -> io::Result<()> {
        let addr = addr
            .to_socket_addrs()
            .map_err(|_| Error::from_raw_os_error(EINVAL))?
            .next()
            .ok_or(Error::from_raw_os_error(EINVAL))?;
        if self.peers.borrow().contains_key(&addr) {
            return Ok(());
        }
        let peer = TcpStream::connect(addr)?;
        self.peers.get_mut().insert(addr, peer);
        let peer = self.peers.get_mut().get_mut(&addr).unwrap();

        let req = Request::Connect(ConnectRequest {
            addr: self.host_address,
        });
        send_packet(peer, &req)?;

        Ok(())
    }

    pub fn request_resource(&mut self, obj: Object) -> Result<Resource, Error> {
        self.peers
            .borrow_mut()
            .iter_mut()
            .find_map(|(_addr, peer)| Self::request_resource_from_peer(peer, &obj).ok())
            .ok_or(Error::new(ErrorKind::NotFound, "Resource not found"))
    }

    fn request_resource_from_peer(peer: &mut TcpStream, obj: &Object) -> Result<Resource, Error> {
        let req = Request::Resource(ResourceRequest { hash: *obj });
        send_packet(peer, &req)?;
        let response = recv_packet::<Response>(peer);
        match response? {
            Response::Resource(resp) => Ok(resp.resource),
            Response::Redirect(_) => todo!(),
            Response::AvailabilityCheck(_) => Err(Error::new(
                ErrorKind::InvalidData,
                "Unexpected value in response to ResourceRequest",
            )),
            Response::Error => Err(Error::new(ErrorKind::NotFound, "Resource not found")),
        }
    }

    fn request_availability_from_peer<T: Iterator<Item = Object>>(
        peer: &mut TcpStream,
        objs: T,
    ) -> Result<Vec<Object>, Error> {
        let req = Request::AvailabilityCheck(AvailabilityCheckRequest {
            hashes: objs.into_iter().collect(),
        });
        send_packet(peer, &req)?;

        match recv_packet::<Response>(peer)? {
            Response::AvailabilityCheck(resp) => Ok(resp.hashes),
            Response::Redirect(_) | Response::Resource(_) => Err(Error::new(
                ErrorKind::InvalidData,
                "Unexpected response to AvailabilityCheckRequest",
            )),
            Response::Error => Err(Error::new(
                ErrorKind::NotFound,
                "Error in AvailabilityCheckRequest",
            )),
        }
    }
}

impl ContentAddressedStore for NetworkClient {
    fn get_resource(&self, object: &Object) -> Option<Resource> {
        self.peers
            .borrow_mut()
            .iter_mut()
            .find_map(|(_addr, peer)| Self::request_resource_from_peer(peer, object).ok())
    }

    fn has(&self, object: &Object) -> bool {
        self.peers.borrow_mut().iter_mut().any(|(_addr, peer)| {
            Self::request_availability_from_peer(peer, [object].iter().cloned().cloned()).is_ok()
        })
    }

    fn accessible_objects(&self) -> Result<Vec<Object>, PathResolutionError> {
        Err(PathResolutionError::new(
            "network client doesn't support listing top-level objects",
        ))
    }
}
