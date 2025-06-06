use core::fmt;
use std::collections::HashMap;
use std::fs::create_dir;
use std::{thread, vec};
use flume::{bounded, Receiver, Sender, unbounded};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, trace, warn, Instrument};
use uuid::Uuid;
use std::fs::File;
use colored::Colorize;
use std::path::{Path, PathBuf};
use interner::shared::{VecStringPool, StringPool};


use crate::error::{MizeError, MizeResult, IntoMizeResult, MizeResultTrait};
use crate::instance::store::Store;
use crate::instance::updater::{ updater_thread, updater_thread_async };
use crate::id::{IntoMizeId, MizeId, Namespace};
use crate::instance::subscription::Subscription;
use crate::item::{Item, ItemData};
use crate::memstore::MemStore;
use crate::instance::updater::Operation;
use crate::{mize_err, Module};
use crate::proto::MizeMessage;

use self::connection::{ConnListener, Connection};
use self::updater::handle_operation;

#[cfg(feature = "async")]
use tokio::runtime::Runtime;
#[cfg(feature = "async")]
use tokio::runtime::Handle;

use core::future::Future;
use std::thread::JoinHandle;

pub mod connection;
pub mod store;
pub mod subscription;
pub mod updater;
pub mod msg_thread;
pub mod module;




#[cfg(test)]
mod tests;

static MSG_CHANNEL_SIZE: usize = 200;

static BUILD_TIME_CONFIG: &str = include_str!(std::env!("MIZE_BUILD_CONFIG"));

/// The Instance type is the heart of the mize system
#[derive(Clone)]
pub struct Instance {
    // a bit a lot of Mutexes isn-it???
    pub store: Arc<Mutex<Box<dyn Store>>>,
    connections: Arc<Mutex<Vec<Connection>>>,
    next_con_id: Arc<Mutex<u64>>,
    subs: Arc<Mutex<HashMap<MizeId, Vec<Subscription>>>>,
    pub modules: Arc<Mutex<HashMap<String, Box<dyn Module + Sync + Send >>>>,
    pub id_pool: Arc<Mutex<VecStringPool>>,
    pub namespace_pool: Arc<Mutex<StringPool>>,

    // the namespace the instance operates in
    pub namespace: Arc<Mutex<Namespace>>,

    // the namespace of the instance itself
    // TODO: set to a random uuid
    pub self_namespace: Arc<Mutex<Namespace>>,
    pub op_tx: Sender<Operation>,
    threads: Arc<Mutex<Vec<(u32, String)>>>,
    next_thread_id: Arc<Mutex<u32>>,
    give_msg_wait: Arc<Mutex<HashMap<MizeId, Vec<Sender<ItemData>>>>>,
    create_msg_wait: Arc<Mutex<Option<Sender<MizeId>>>>,

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


        let id_pool = Arc::new(Mutex::new(VecStringPool::default()));
        let namespace_pool_raw = StringPool::default();
        let connections = Arc::new(Mutex::new(Vec::new()));
        let subs = Arc::new(Mutex::new(HashMap::new()));
        let (op_tx, op_rx) = unbounded();
        let give_msg_wait = Arc::new(Mutex::new(HashMap::new()));
        let create_msg_wait = Arc::new(Mutex::new(None));
        let namespace = Arc::new(Mutex::new(Namespace ( namespace_pool_raw.get("mize.default.namespace") )));
        let self_namespace = Arc::new(Mutex::new(Namespace ( namespace_pool_raw.get("mize.default.namespace") )));


        let mut instance = Instance { 
            store: Arc::new(Mutex::new(Box::new(MemStore::new()))),
            connections, subs, id_pool,
            namespace, self_namespace, op_tx,
            namespace_pool: Arc::new(Mutex::new(namespace_pool_raw)),
            modules: Arc::new(Mutex::new(HashMap::new())),
            threads: Arc::new(Mutex::new(Vec::new())),
            next_thread_id: Arc::new(Mutex::new(0)),
            next_con_id: Arc::new(Mutex::new(1)),
            give_msg_wait, create_msg_wait,

            #[cfg(feature = "async")]
            runtime: Arc::new(Mutex::new(Runtime::new().mize_result_msg("Could not create async runtime")?)),
        };



        #[cfg(feature = "os-target")]
        {
            let instance_clone = instance.clone();
            let op_rx_clone = op_rx.clone();
            let closure = move || updater_thread(op_rx_clone, &instance_clone);
            instance.spawn("updater_thread", closure)?;

            let instance_clone_two = instance.clone();
            let closure_two = move || updater_thread(op_rx, &instance_clone_two);
            instance.spawn("updater_thread", closure_two)?;
        }


        // set up async update "threads" when using wasm
        #[cfg(feature = "wasm-target")]
        {
            let instance_clone = instance.clone();
            let op_rx_clone = op_rx.clone();
            wasm_bindgen_futures::spawn_local(updater_thread_async(op_rx_clone, instance_clone));

            let instance_clone_two = instance.clone();
            wasm_bindgen_futures::spawn_local(updater_thread_async(op_rx, instance_clone_two));
        }

        // will move the msg stuff into it's own thread
        // like this it can deadlock.... if a msg waits on an operation to complete
        //let msg_instance_clone = instance.clone();
        //let msg_closure = move || msg_thread(op_rx, instance_clone);
        //instance.spawn("msg_thread", closure)?;


        // load the config from build time
        let config = ItemData::from_toml(BUILD_TIME_CONFIG)?;
        instance.set_blocking("0", config);


        return Ok(instance);
    }

    pub fn new() -> MizeResult<Instance> {
        trace!("[ {} ] Instance::new()", "CALL".yellow());


        let mut instance = Instance::empty()?;


        instance.init();


        debug!("instance inited with config: {}", instance.get("0/config")?.as_data_full()?);


        return Ok(instance);
    }

    pub fn init(&mut self) -> MizeResult<()> {

        // platform specific init code
        crate::platform::any::instance_init(self)?;

        // end of platform specific init code


        // load the modules, ad specified in the load_modules config
        match self.get("0/config/load_modules")?.value_string() {
            Ok(modules_to_load) => {
                for module in modules_to_load.split(" ") {
                    self.load_module(module)?;
                }
            },
            Err(err) => {
                // no load_modules option is set
            }
        }



        debug!("INSTANCE INIT DONE");
        Ok(())
    }

    pub fn with_config(config: ItemData) -> MizeResult<Instance> {
        trace!("[ {} ] Instance::with_config()", "CALL".yellow());
        trace!("config: {}", config);

        let mut instance = Instance::empty()?;

        instance.set_blocking("0", config);

        instance.init()?;

        Ok(instance)
    }

    pub fn migrate_to_store(&self, new_store: Box<dyn Store>) -> MizeResult<()> {
        info!("MIGRATING");
        let mut old_store = self.store.lock()?;

        let id = self.id_from_string("0".to_owned())?;
        let inst_data = old_store.get_value_data_full(id.clone())?;
        new_store.set(id, inst_data.to_owned())?;

        for id in old_store.id_iter()? {
            let data = old_store.get_value_data_full(self.id_from_string(id?)?)?;

            let id_of_new_store = new_store.new_id()?;
            new_store.set(self.id_from_string(id_of_new_store)?, data.to_owned())?;
        };

        *old_store = new_store;

        Ok(())
    }

    pub fn new_item(&self) -> MizeResult<Item> {
        if self.get_namespace()? != self.get_self_namespace()? {
            // need to send create msg and wait for it
            let mut connection = self.get_connection_by_ns(self.get_namespace()?)?;

            let msg = MizeMessage::new_create(connection.id);

            connection.send(msg)?;

            let (tx, rx) = bounded::<MizeId>(1);

            let mut msg_wait_inner = self.create_msg_wait.lock()?;
            *msg_wait_inner = Some(tx);
            drop(msg_wait_inner);

            let id = rx.recv()?;
            println!("new_item namespace: {:?}", id.namespace());

            return Ok(Item::new(id, self));
        }

        let store_inner = self.store.lock()?;
        let id = self.id_from_string(store_inner.new_id()?)?;
        return Ok(Item::new(id, self));
    }

    pub fn get<I: IntoMizeId>(&self, id: I) -> MizeResult<Item> {
        let id = id.to_mize_id(self)?;
        return Ok(Item::new(id, &self));
    }

    pub fn set<I: IntoMizeId, V: Into<ItemData>>(&self, id: I, value: V) -> MizeResult<()> {
        let id = id.to_mize_id(self)?;
        self.op_tx.send(Operation::Set(id, value.into(), None));
        Ok(())
    }
    
    pub fn set_blocking<I: IntoMizeId, V: Into<ItemData>>(&self, id: I, value: V) -> MizeResult<()> {
        handle_operation(&mut Operation::Set(id.to_mize_id(self)?, value.into(), None), self)?;
        Ok(())
    }

    pub fn sub<I: IntoMizeId>(&self, id: I, sub: Subscription) -> MizeResult<()> {
        let mut subs_inner = self.subs.lock()?;
        let id = id.to_mize_id(self)?;
        match subs_inner.get_mut(&id) {
            Some(vec) => {
                vec.push(sub);
            },
            None => {
                subs_inner.insert(id.clone(), vec![sub]);
            }
        }

        // if we are not the owner of this item, send a sub msg to them
        if !self.we_are_namespace()? {
            let con = self.get_connection_by_ns(id.namespace())?;
            let msg = MizeMessage::new_sub(id, con.id);
            con.send(msg)?;
        }

        Ok(())
    }

    pub fn new_id<T: IntoMizeId>(&self, value: T) -> MizeResult<MizeId> {
        value.to_mize_id(self)
    }

    pub fn id_from_string(&self, string: String) -> MizeResult<MizeId> {
        let vec_string: Vec<String> = string.split("/").map(|v| v.to_owned()).filter(|el| el.as_str() != "").collect();
        self.id_from_vec_string(vec_string)
    }

    pub fn id_from_vec_string(&self, mut vec_string: Vec<String>) -> MizeResult<MizeId> {
        let id_pool_inner = self.id_pool.lock()?;
        let namespace_inner = self.namespace.lock()?;
        let first_el = vec_string.first_mut().ok_or(mize_err!("MizeId was empty"))?;

        let id = if first_el.contains(":") { // first el is a namespace + store_part
            let new_first_el = first_el.clone();
            let vec: Vec<&str> = new_first_el.split(":").collect();
            let ns_part = vec.iter().nth(0).ok_or(mize_err!("should really not happen"))?;
            let store_part = vec.iter().nth(1).ok_or(mize_err!("mizeid was like 'namespace:/hi', why are you doing that"))?;
            *first_el = store_part.to_owned().to_owned();

            MizeId { path: id_pool_inner.get(vec_string), namespace: self.namespace_from_string(ns_part.to_owned().to_owned())? }
        } else {

            MizeId { path: id_pool_inner.get(vec_string), namespace: namespace_inner.clone() }
        };
        trace!("new MizeId made: {:?}", id);

        Ok(id)
    }

    pub fn load_module(&mut self, name: &str) -> MizeResult<()> {
        crate::platform::any::load_module(self, name, None)
    }

    pub fn fetch_module(&mut self, name: &str) -> MizeResult<String> {
        // platform specific way to fetch a module
        crate::platform::any::fetch_module(self, name)
    }

    pub fn load_module_at(&mut self, name: &str, path: String) -> MizeResult<()> {
        // platform specific init code
        crate::platform::any::load_module(self, name, Some(path))
    }

    pub fn get_module(&mut self, name: &str) -> MizeResult<Box<dyn Module + Send + Sync>> {
        let inner = self.modules.lock()?;

        let module = inner.get(name).ok_or(mize_err!("Couldn't get_module('{name}')"))?.clone_module();

        Ok(module)
    }

    pub fn namespace_from_string(&self, ns_str: String) -> MizeResult<Namespace> {
        let namespace_pool_inner = self.namespace_pool.lock()?;
        let namespace = Namespace ( namespace_pool_inner.get(ns_str) );
        Ok(namespace)
    }

    pub fn set_namespace(&self, ns: Namespace) -> MizeResult<()> {
        let mut namespace_inner = self.namespace.lock()?;
        *namespace_inner = ns;

        Ok(())
    }

    pub fn get_namespace(&self) -> MizeResult<Namespace> {
        let mut namespace_inner = self.namespace.lock()?;
        return Ok(namespace_inner.clone());
    }

    pub fn get_self_namespace(&self) -> MizeResult<Namespace> {
        let mut self_namespace_inner = self.self_namespace.lock()?;
        return Ok(self_namespace_inner.clone());
    }

    pub fn we_are_namespace(&self) -> MizeResult<bool> {
        Ok(self.get_namespace()? == self.get_self_namespace()?)
    }

    pub fn add_listener<T: ConnListener +'static>(&mut self, listener: T) -> MizeResult<()> {
        let mut instance_clone = self.clone();
        self.spawn("some_listener", move || listener.listen(instance_clone));
        Ok(())
    }

    pub fn new_connection(&self, tx: Sender<MizeMessage>) -> MizeResult<u64> {
        let mut conn_inner = self.connections.lock()?;
        let mut next_con_id = self.next_con_id.lock()?;
        let old_next_con_id = *next_con_id;

        let connection = Connection { id: next_con_id.to_owned(), tx, ns: None};
        conn_inner.push(connection);
        *next_con_id += 1;
        Ok(old_next_con_id)
    }

    pub fn new_connection_join_namespace(&self, tx: Sender<MizeMessage>) -> MizeResult<u64> {
        let conn_id = self.new_connection(tx)?;

        let ns_of_peer_str = self.get(format!("inst/con_by_id/{}/peer/0/config/namespace", conn_id))?.value_string()?;
        let ns_of_peer = self.namespace_from_string(ns_of_peer_str)?;

        self.connection_set_namespace(conn_id, ns_of_peer.clone());
        self.set_namespace(ns_of_peer);

        Ok(conn_id)
    }

    pub fn connection_set_namespace(&self, conn_id: u64, namespace: Namespace) -> MizeResult<()> {
        let mut connection = self.get_connection(conn_id)?;
        connection.ns = Some(namespace);
        self.set_connection(conn_id, connection);
        Ok(())
    }

    pub fn got_msg(&self, msg: MizeMessage) -> MizeResult<()> {
        Ok(self.op_tx.send(Operation::Msg(msg))?)
    }

    pub fn report_err(&self, err: MizeError) {
        err.log();
    }

    fn set_connection(&self, conn_id: u64, new_connection: Connection) -> MizeResult<()> {
        let mut conn_inner = self.connections.lock()?;

        for connection in conn_inner.iter_mut() {
            if connection.id == conn_id {
                *connection = new_connection;
                return Ok(());
            }
        }

        return Err(mize_err!("Connection with id {} not known to instance", conn_id));
        Ok(())
    }

    pub fn get_connection(&self, conn_id: u64) -> MizeResult<Connection> {
        let mut conn_inner = self.connections.lock()?;

        for connection in conn_inner.iter() {
            if connection.id == conn_id {
                return Ok(connection.clone());
            }
        }

        return Err(mize_err!("Connection with id {} not known to instance", conn_id));
    }

    pub fn get_connection_by_ns(&self, ns: Namespace) -> MizeResult<Connection> {
        let mut conn_inner = self.connections.lock()?;

        for connection in conn_inner.iter() {
            if connection.ns == Some(ns.clone()) {
                return Ok(connection.clone());
            }
        }

        return Err(mize_err!("Connection with namespace {} not known to instance", ns.as_string()));
    }

    pub fn spawn(&mut self, name: &str, func: impl FnOnce() -> MizeResult<()> + Send + 'static) -> MizeResult<()> {

        let mut threads_inner = self.threads.lock()?;
        let mut next_thread_id = self.next_thread_id.lock()?;

        threads_inner.push((*next_thread_id, name.to_owned()));

        let my_thread_id_no_mutex_guard = *next_thread_id;
        let thread_mutex = self.threads.clone();
        let name_to_move = name.to_owned();
        let to_spawn = move || -> MizeResult<()> {
            debug!("spawning thread: {}", name_to_move);
            let my_thread_id = my_thread_id_no_mutex_guard;

            func()?;

            let mut threads_inner = thread_mutex.lock()?;
            *threads_inner = threads_inner.clone().into_iter().filter(|el| match el {
                (my_thread_id, _) => false,
                (_, _) => true,
            }).collect();
            debug!("thread '{}' stopped", name_to_move);
            Ok(())
        };

        *next_thread_id += 1;

        #[cfg(feature = "os-target")]
        thread::spawn(move || to_spawn());

        #[cfg(feature = "wasm-target")]
        {
            //console_log!("in instance::spawn with wasm target")
        }
        //NOT WELL SUPPORTED
        //crate::platform::wasm::wasm_spawn(to_spawn)?;

        Ok(())
    }

    pub fn give_msg_wait(&self, id: MizeId) -> MizeResult<ItemData> {

        let mut give_msg_wait_inner = self.give_msg_wait.lock()?;

        let (tx, rx) = bounded::<ItemData>(1);

        let vec = match give_msg_wait_inner.get_mut(&id) {
            Some(vec) => vec,
            None => {
                give_msg_wait_inner.insert(id.clone(), Vec::new());
                give_msg_wait_inner.get_mut(&id).unwrap()
            }
        };

        vec.push(tx);

        // so that another thread can also give_msg_wait(), while we wait in the recv() of rx
        drop(vec);
        drop(give_msg_wait_inner);

        let data = rx.recv()?;

        return Ok(data);
    }


    #[cfg(feature = "async")]
    pub fn spawn_async<F: Future<Output = impl Send + Sync + 'static> + Send + Sync + 'static>(&mut self, name: &str, func: F) -> MizeResult<()> {
        let mut threads_inner = self.threads.lock()?;
        let mut next_thread_id = self.next_thread_id.lock()?;

        threads_inner.push((*next_thread_id, name.to_owned()));
        let runtime_inner = self.runtime.lock().unwrap();
        runtime_inner.spawn(func);

        *next_thread_id += 1;

        Ok(())
    }


    #[cfg(feature = "async")]
    pub fn async_get_handle(&self) -> Handle {
        let runtime_inner = self.runtime.lock().unwrap();
        let handle = runtime_inner.handle().to_owned();
        handle
    }

    #[cfg(feature = "async")]
    pub fn spawn_async_blocking<F: Future<Output = impl Send + Sync + 'static> + Send + Sync + 'static>(&mut self, name: &str, func: F) -> F::Output {

        let mut threads_inner = self.threads.lock().expect("mutex lock failed in spawn_async_blocking");
        let mut next_thread_id = self.next_thread_id.lock().expect("mutex lock failed in spawn_async_blocking");

        threads_inner.push((*next_thread_id, name.to_owned()));
        let runtime_inner = self.runtime.lock().unwrap();
        let handle = runtime_inner.handle().to_owned();
        *next_thread_id += 1;

        drop(runtime_inner);
        drop(threads_inner);
        drop(next_thread_id);

        let result = handle.block_on(func);


        return result;
    }

    pub fn wait(&self) {
        info!("Instance main thread waiting");
        loop {
            thread::sleep_ms(10000000)
        }
    }


    pub fn report_error(err: MizeError) {
        err.log();
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mize Instance with subs: {:?}", self.subs,)
    }
}

