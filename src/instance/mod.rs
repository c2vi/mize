use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::create_dir;
use colored::Colorize;
use log::{trace, debug, info, warn, error};
use serde_json::json;
use tokio::net::{UnixListener, UnixStream};
use uuid::Uuid;
use std::fs::File;
use std::path::Path;
use daemonize::Daemonize;
use nix::unistd::Pid;

use crate::error::{MizeError, MizeResult, IntoMizeResult, MizeResultTrait};
use crate::itemstore::{Itemstore, self};
use crate::item::{Item, MizeId, ItemRef};

pub mod peer;
pub mod connection;

pub use connection::Connection;

static UNIX_SOCKET_CHANNEL_SIZE: usize = 200;
static WEB_SOCKET_CHANNEL_SIZE: usize = 200;
static TCP_SOCKET_CHANNEL_SIZE: usize = 200;
static UPDATE_CHANNEL_SIZE: usize = 200;


/// The Instance type is the heart of the mize system
pub struct Instance {
    store: Itemstore,
    peers: Vec<Connection>,
    realm_instance: Option<(RealmId, Vec<Connection>)>,
}

pub enum RealmId {
    Uuid(Uuid),
    Local(Vec<String>),
    Tld(Vec<String>),
}

impl Instance {
    pub async fn new(instance_path: PathBuf) -> MizeResult<Instance> {
        trace!("[ {} ] Instance::new()", "CALL".yellow());
        trace!("[ {} ] instance_path: {:?}", "ARG".yellow(), instance_path);

        // if path does not exist, create it
        if instance_path.exists() == false {
            debug!("instance path \"{}\" does not exist, so creating it", instance_path.clone().display());
            File::create(&instance_path)
                .mize_result_msg(format!("Could not create Instance at: {}", &instance_path.display())).critical();
        }

        // if a daemon is running, return instance with memory store
        // with realm_instance at that daemon
        let mut instance = match find_running_ademon(instance_path.clone()).await {
            Ok(con) => {
                Instance {
                    store: Itemstore::Memory(itemstore::new_memory_store()?),
                    peers: vec![con],
                    realm_instance: None,
                }
            },

            // else open the itemstore
            Err(err) => {
                Instance {
                    store: Itemstore::Folder(itemstore::new_folder_store(instance_path)?),
                    peers: vec![],
                    realm_instance: None,
                }
            },
        };

        if !instance.store.has_item(MizeId::main(1)).await? {
            instance.store.create_item_from_json(json!({
                "item": {
                    "kind": "instance",
                },
                "daemon": {
                    "log": "",
                    "pid": "",
                    "item": {
                        "kind": "instance/daemon",
                    },
                },
            }));
        };

        return Ok(instance);

    }

    pub async fn start_deamon(mut self) -> MizeResult<()> {
        trace!("[ {} ] Instance::start_deamon()", "CALL".yellow());
        // daemonize
        let stdout = self.get_item(MizeId::main(1).join("daemon").join("log")).await?.value_as_file()?;

        let pid_file = self.get_item(MizeId::main(1).join("daemon").join("pid")).await?.value_as_path()?;

        let daemonize = Daemonize::new()
            .pid_file(pid_file)
            //.chown_pid_file(true)      // is optional, see `Daemonize` documentation
            //.working_directory("/tmp") // for default behaviour.
            //.user("nobody")
            //.group("daemon") // Group name
            //.group(2)        // or group id.
            //.umask(0o777)    // Set umask, `0o027` by default.
            .stdout(stdout);  // Redirect stdout to `/tmp/daemon.out`.
            //.privileged_action(|| "Executed before drop privileges");

        daemonize.start().mize_result_msg("Error daemonizing")?;

        info!("Succesfuly Daemonized");

        // listen on unix socket
        let sock_path = self.get_item(MizeId::main(1).join("daemon").join("sock")).await?.value_as_path()?;
        let listener = UnixListener::bind(&sock_path)
            .mize_result_msg(format!("Could not bind to unix Socket at: {}", sock_path.display())).critical();
        loop {
            match listener.accept().await {
                Ok((unix_sock, _addr)) => {
                    self.peers.push(Connection::from_unix(unix_sock))
                },
                Err(e) => { MizeError::new().msg("New Peer failed").msg(format!("{}", e)).log(); },
            };
        }
    }

    pub async fn get_daemon_pid(self) -> MizeResult<Pid> {
        trace!("[ {} ] Instance::get_daemon_pid()", "CALL".yellow());
        todo!()
    }

    pub async fn daemon_is_running(&self) -> MizeResult<bool> {
        trace!("[ {} ] Instance::daemon_is_running()", "CALL".yellow());
        let sock_path = self.get_item(MizeId::main(1).join("daemon").join("sock")).await?.value_as_path()?;
        match find_running_ademon(sock_path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }


    // item funcs
    pub async fn get_item(&self, id: MizeId) -> MizeResult<Item> {
        self.store.get_item(id).await
    }
    pub async fn set_item(&mut self, item: Item) -> MizeResult<()> {
        self.store.set_item(item).await
    }
    pub async fn create_item(&mut self) -> MizeResult<MizeId> {
        self.store.create_item().await
    }

    //pub fn update_item(self) -> MizeResult<Item> {
        //todo!()
    //}
    //pub fn on_item_update(self) -> MizeResult<Item> {
        //todo!()
    //}
}

async fn find_running_ademon(path: PathBuf) -> MizeResult<Connection> {
        match UnixStream::connect(path).await {
            Ok(val) => Ok(Connection::from_unix(val)),
            Err(err) => Err(MizeError::new().msg("No Daemon found to connect")),
        }
}


fn handle_connection() {
    todo!()
}

/*
fn get_instance_type() -> MizeResult<Item> {

    let mut types_item = Item::new("types");

    // instance
    let instance_str = include_str!("../../types/mize/instance/item.json");
    let instance = Item::new("type").load_json(instance_str)?;
    types_item.set("instance", instance);

    return Ok(types_item);
}
*/




