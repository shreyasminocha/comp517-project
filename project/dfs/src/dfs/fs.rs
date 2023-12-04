use std::{
    collections::BTreeMap,
    io::{self},
    net::{SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex},
};

use crate::{
    cas::{error::PathResolutionError, object::Object, resource::Resource, ContentAddressedStore},
    network::{fs::NetworkClient, server::spawn_server},
    store::fs::LocalStore,
};

pub struct Filesystem {
    address: SocketAddr,
    store: Arc<Mutex<LocalStore>>,
    client: Arc<Mutex<NetworkClient>>,
}

impl Filesystem {
    pub fn new<A>(address: A) -> Self
    where
        A: ToSocketAddrs + Send + 'static,
    {
        let address = address.to_socket_addrs().unwrap().next().unwrap();
        Self {
            address,
            store: Arc::new(Mutex::new(LocalStore::new())),
            client: Arc::new(Mutex::new(NetworkClient::new(address))),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        spawn_server(self.store.clone(), self.client.clone(), self.address);
        Ok(())
    }

    pub fn add_peer<A: ToSocketAddrs>(&mut self, addr: A) -> io::Result<()> {
        println!("addpeer called");
        let x = self.client.lock().unwrap().add_peer(addr);
        println!("addpeer done");
        x
    }

    pub fn create_file(&mut self, contents: &[u8]) -> Object {
        self.store.lock().unwrap().create_file(contents)
    }

    pub fn create_directory(&mut self, contents: BTreeMap<String, Object>) -> Object {
        self.store.lock().unwrap().create_directory(contents)
    }
}

impl ContentAddressedStore for Filesystem {
    fn get_resource(&self, object: &Object) -> Option<Resource> {
        let attempted_resource = self.store.get(object);

        if attempted_resource.is_some() {
            return attempted_resource;
        }

        let attempted_resource: Option<Resource> = self.client.lock().unwrap().get(object);

        if let Some(resource) = &attempted_resource {
            self.store.lock().unwrap().add_resource(resource.clone());
        }

        attempted_resource
    }

    fn has(&self, object: &Object) -> bool {
        // TODO: query network
        self.store.has(object)
    }

    fn accessible_objects(&self) -> Result<Vec<Object>, PathResolutionError> {
        self.store.accessible_objects()
    }
}
