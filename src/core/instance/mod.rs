use core::fmt;
use std::collections::HashMap;
use std::fs::create_dir;
use std::thread::JoinHandle;
use std::{thread, vec};
use crossbeam::channel::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tracing::{trace, debug, info, warn, error};
use uuid::Uuid;
use std::fs::File;
use colored::Colorize;
use std::path::Path;
use interner::shared::{VecStringPool, StringPool};


use crate::error::{MizeError, MizeResult, IntoMizeResult, MizeResultTrait};
use crate::instance::store::Store;
use crate::instance::updater::updater_thread;
use crate::id::{IntoMizeId, MizeId, Namespace};
use crate::instance::subscription::Subscription;
use crate::item::{Item, ItemData};
use crate::memstore::MemStore;
use crate::instance::updater::Operation;
use crate::mize_err;
use crate::proto::MizeMessage;

use self::connection::{ConnListener, Connection};

#[cfg(feature = "async")]
use tokio::runtime::Runtime;
use core::future::Future;

pub mod connection;
pub mod store;
pub mod subscription;
pub mod updater;
pub mod msg_thread;

static MSG_CHANNEL_SIZE: usize = 200;

/// The Instance type is the heart of the mize system
#[derive(Clone)]
pub struct Instance {
    // question vec of stores???
    // would mean replication logic and such things is handeled by the instance
    // i think it would be better thogh to then implement a ReplicatedStore
    pub store: Box<dyn Store>,
    connections: Arc<Mutex<Vec<Connection>>>,
    next_con_id: Arc<Mutex<u64>>,
    subs: Arc<Mutex<HashMap<MizeId, Subscription>>>,
    pub id_pool: Arc<Mutex<VecStringPool>>,
    pub namespace_pool: Arc<Mutex<StringPool>>,
    pub namespace: Namespace,
    op_tx: channel::Sender<Operation>,
    context: Vec<MizeId>,
    threads: Vec<String>,
    #[cfg(feature = "async")]
    pub runtime: Arc<Mutex<Runtime>>,
}

pub struct InstanceRef {
    inner: Arc<Mutex<Instance>>,
}

pub struct InstanceAsync {
    inner: Arc<Mutex<Instance>>,
}


impl Instance {
    pub fn empty() -> MizeResult<Instance> {
        let store = MemStore::new();
        let id_pool = Arc::new(Mutex::new(VecStringPool::default()));
        let namespace_pool_raw = StringPool::default();
        let connections = Arc::new(Mutex::new(Vec::new()));
        let subs = Arc::new(Mutex::new(HashMap::new()));
        let namespace = Namespace ( namespace_pool_raw.get("mize.default.namespace") );
        let (op_tx, op_rx) = channel::unbounded();

        let mut instance = Instance { 
            store: Box::new(store), 
            connections, subs, id_pool,
            namespace, op_tx,
            namespace_pool: Arc::new(Mutex::new(namespace_pool_raw)),
            context: vec![],
            threads: vec![],
            next_con_id: Arc::new(Mutex::new(1)),

            #[cfg(feature = "async")]
            runtime: Arc::new(Mutex::new(Runtime::new().mize_result_msg("Could not create async runtime")?)),
        };

        let instance_clone = instance.clone();
        let closure = move || updater_thread(op_rx, instance_clone);
        instance.spawn("updater_thread", closure)?;

        // will probably move the msg stuff into it's own thread
        //let msg_instance_clone = instance.clone();
        //let msg_closure = move || msg_thread(op_rx, instance_clone);
        //instance.spawn("msg_thread", closure)?;


        return Ok(instance);
    }

    pub fn new() -> MizeResult<Instance> {
        trace!("[ {} ] Instance::new()", "CALL".yellow());

        let mut instance = Instance::empty()?;

        instance.init();

        debug!("instance inited with config: {}", instance.get("0/config")?.as_data_full()?);

        return Ok(instance);
    }

    fn init(&mut self) -> MizeResult<()> {

        // platform specific init code
        if cfg!(feature = "os-target") { // on os platforms
            crate::platform::os::os_instance_init(self)?
        }

        // end of platform specific init code

        Ok(())
    }

    pub fn with_config(config: ItemData) -> MizeResult<Instance> {
        trace!("[ {} ] Instance::with_config()", "CALL".yellow());
        trace!("config: {}", config);
        let mut instance = Instance::empty()?;
        instance.set("0", config.clone());
        instance.init()?;

        // set it again, so that the passed config data has presidence over anything the init would set
        debug!("overwriting instance config again with the one passed to Instance::with_config()");
        instance.set("0", config);

        debug!("instance inited with conifg: {}", instance.get("0/config")?);

        Ok(instance)
    }

    pub fn migrate_to_store(&mut self, new_store: Box<dyn Store>) -> MizeResult<()> {
        println!("MIGRATING");
        let old_store = &self.store;

        let id = self.id_from_string("0".to_owned())?;
        let inst_data = self.store.get_value_data_full(id.clone())?;
        new_store.set(id, inst_data.to_owned())?;

        for id in old_store.id_iter()? {
            let data = self.store.get_value_data_full(self.id_from_string(id?)?)?;

            let id_of_new_store = new_store.new_id()?;
            new_store.set(self.id_from_string(id_of_new_store)?, data.to_owned())?;
        };

        self.store = new_store;

        Ok(())
    }

    pub fn new_item(&self) -> MizeResult<Item> {
        let id = self.id_from_string(self.store.new_id()?)?;
        return Ok(Item::new(id, &self));
    }

    pub fn get<I: IntoMizeId>(&self, id: I) -> MizeResult<Item> {
        let id = id.to_mize_id(self)?;
        return Ok(Item::new(id, &self));
    }

    pub fn set<I: IntoMizeId, V: Into<ItemData>>(&mut self, id: I, value: V) -> MizeResult<()> {
        let id = id.to_mize_id(self)?;
        self.op_tx.send(Operation::Set(id, value.into()));
        Ok(())
    }

    pub fn new_id<T: IntoMizeId>(&self, value: T) -> MizeResult<MizeId> {
        value.to_mize_id(self)
    }

    pub fn id_from_string(&self, string: String) -> MizeResult<MizeId> {
        let vec_string: Vec<String> = string.split("/").map(|v| v.to_owned()).collect();
        let id_pool_inner = self.id_pool.lock()?;

        let id = MizeId { path: id_pool_inner.get(vec_string), namespace: self.namespace.clone() };

        Ok(id)
    }

    pub fn id_from_vec_string(&self, vec_string: Vec<String>) -> MizeResult<MizeId> {
        let id_pool_inner = self.id_pool.lock()?;
        let id = MizeId { path: id_pool_inner.get(vec_string), namespace: self.namespace.clone() };
        Ok(id)
    }

    pub fn namespace_from_string(&self, ns_str: String) -> MizeResult<Namespace> {
        let namespace_pool_inner = self.namespace_pool.lock()?;
        let namespace = Namespace ( namespace_pool_inner.get(ns_str) );
        Ok(namespace)
    }

    pub fn add_listener<T: ConnListener +'static>(&mut self, listener: T) -> MizeResult<()> {
        let mut instance_clone = self.clone();
        self.spawn("some_listener", move || listener.listen(instance_clone));
        Ok(())
    }

    pub fn new_connection(&mut self, rx: Receiver<MizeMessage>, tx: Sender<MizeMessage>) -> MizeResult<u64> {
        let mut conn_inner = self.connections.lock()?;
        let mut next_con_id = self.next_con_id.lock()?;

        let connection = Connection { id: next_con_id.to_owned(), rx, tx};
        conn_inner.push(connection);
        *next_con_id += 1;
        Ok(next_con_id.to_owned())
    }

    pub fn get_connection(&mut self, conn_id: u64) -> MizeResult<Connection> {
        let mut conn_inner = self.connections.lock()?;

        for connection in conn_inner.iter() {
            if connection.id == conn_id {
                return Ok(connection.clone());
            }
        }

        return Err(mize_err!("Connection with id {} not known to instance", conn_id));
    }

    pub fn spawn(&mut self, name: &str, func: impl FnOnce() -> MizeResult<()> + Send + 'static) -> MizeResult<()> {
        self.threads.push(name.to_owned());
        thread::spawn(move || func());
        Ok(())
    }

    #[cfg(feature = "async")]
    pub fn spawn_async<F: Future<Output = impl Send + Sync + 'static> + Send + Sync + 'static>(&mut self, name: &str, func: F) {
        self.threads.push(name.to_owned());
        let runtime_inner = self.runtime.lock().unwrap();
        runtime_inner.spawn(func);
    }

    pub fn wait(&self) {
        loop {
            thread::sleep_ms(10000000)
        }
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mize Instance with subs: {:?}", self.subs,)
    }
}

