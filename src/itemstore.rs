use log::{trace, debug, info, warn, error};
use std::fs::{self, File};
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::io::{Read, Write};
use colored::Colorize;

use crate::item::{Item, MizeId, ItemCbor, ItemDescriptor};
use crate::error::{MizeError, MizeResult, IntoMizeResult};

pub enum Itemstore {
    Folder(FolderItemstore),
    Memory(MemoryItemstore),
}

pub struct FolderItemstore {
    path: PathBuf,
    next_id: u64,
}

pub struct MemoryItemstore {
    inner: HashMap<MizeId, Item>,
    next_id: u64,
}


///////////////////////////////////////////////////////////////////////////////////////////


pub fn new_memory_store() -> MizeResult<MemoryItemstore> {
    Ok(MemoryItemstore {
        inner: HashMap::new(),
        next_id: 100,
    })
} 

pub fn new_folder_store(path: PathBuf) -> MizeResult<FolderItemstore> {
    if path.join("store_open").exists() {
        let pid_str = fs::read_to_string(path.join("store_open"))
            .mize_result_msg("Could not read the store_open file to check which process has the store opened")?;
        let pid = pid_str.parse::<u32>()
            .mize_result_msg("Could not Parse the content of the store_open file into a u32")?;

        if Path::new("/proc").join(format!("{}", pid)).exists() {
            return Err(MizeError::new().msg(
                    format!("the store at {} is already open by process with pid: {}", path.display(), pid)
                ));
        } else {
            warn!("An Instance was not closed properly. (store_open file contained a not running PID)");
            std::fs::remove_file(path.join("store_open"))
                .mize_result_msg("Could not remove the store_open file")?;
        }
    }

    let mut store_open_file = File::create(path.join("store_open"))
        .mize_result_msg("Could not create the store_open file")?;

    store_open_file.write_all(format!("{}", std::process::id()).as_bytes())
        .mize_result_msg("Could not write my pid to the store_open file")?;

    // if no store_next_id exists write it with the default 100
    let mut next_id: u64 = 100; // as a default
    if !path.join("store_next_id").exists() {
        fs::write(path.join("store_next_id"), format!("{}", next_id).as_bytes())
            .mize_result_msg("Could not write the default next_id to store_next_id")?;
    } else {
        let next_id_string = fs::read_to_string(path.join("store_next_id"))
            .mize_result_msg("Could not read store_next_id of this store")?;

        next_id = next_id_string.parse()
            .mize_result_msg("Could not parse contents of store_next_id into u64")?;
    }

    return Ok(FolderItemstore{ path, next_id});

} 


///////////////////////////////////////////////////////////////////////////////////////////


impl FolderItemstore {
    pub async fn write_next_id(&self) -> MizeResult<()> {
        fs::write(self.path.join("store_next_id"), format!("{}", self.next_id).as_bytes())
            .mize_result_msg(format!("Could not write the next_id: \"{}\" to \"{}\"", self.next_id, self.path.join("store_next_id").display()))?;
        return Ok(());
    }
}


///////////////////////////////////////////////////////////////////////////////////////////


impl MemoryItemstore {
    // item funcs
    async fn get_item(&self, id: MizeId) -> MizeResult<Item> {
        match self.inner.get(&id) {
            None => {
                Err(MizeError::new().msg(format!("there was no item with id ({}) in this store", id)))
            },
            Some(item) => {
                Ok(Item::Cbor(item.as_cbor()?))
            },
        }
    }

    async fn set_item(&mut self, item: Item) -> MizeResult<()> {
        self.inner.insert(item.id()?, item)
            .ok_or(MizeError::new().msg("Could not set an item in a MemoryStore"))?;
        return Ok(());
    }
    

    async fn create_item(&mut self) -> MizeResult<MizeId> {
        // advance the next_id
        self.next_id += 1;
        return Ok(MizeId::main(self.next_id))
    }

    //fn update_item(self) -> MizeResult<I>;

    // other funcs
    async fn close(&self) -> MizeResult<()> {
        return Ok(());
    }

    async fn next_id(&self) -> u64 {
        self.next_id
    }

    async fn has_item(&self, id: MizeId) -> MizeResult<bool> {
        Ok(self.inner.contains_key(&id))
    }
}

impl FolderItemstore {
    // item funcs
    async fn get_item(&self, id: MizeId) -> MizeResult<Item> {
        let id_string = format!("{}", id.main);
        let mut final_path = self.path.join("store");
        if id_string.len() < 2 {
            // use path 00 for ids under 10
            final_path.push("00")
        } else {
            // will not panic, because of prior check if string is more than two long
            final_path.push(id_string[..2].to_owned());
        };

        return Ok(Item::Descriptor(ItemDescriptor { path: final_path, id}));
    }

    async fn set_item(&self, item: Item) -> MizeResult<()> {
        item.write_to_path(self.get_path_of_item(&item.id()?)?)
    }

    async fn create_item(&mut self) -> MizeResult<MizeId> {
        // advance the next_id
        self.next_id += 1;
        match self.write_next_id().await {
            // on error revert to old next_id
            Ok(_) => {},
            Err(err) => {
                self.next_id -= 1;
                return Err(err);
            },
        }
        return Ok(MizeId::main(self.next_id));
    }

    //fn update_item(self) -> MizeResult<Item>;

    // other funcs
    async fn close(&self) -> MizeResult<()> {
        if self.path.join("store_open").exists() {
            std::fs::remove_file(self.path.join("store_open"))
                .mize_result_msg("Could not remove the store_open file")?;
        }
        return Ok(());
    }

    async fn next_id(&self) -> u64 {
        self.next_id
    }
    async fn has_item(&self, id: MizeId) -> MizeResult<bool> {
        Ok(self.get_path_of_item(&id)?.exists())
    }

    fn get_path_of_item(&self, id: &MizeId) -> MizeResult<PathBuf> {
        // get path of item
        let id_string = format!("{}", id.main);
        let mut final_path = self.path.join("store");
        if id_string.len() < 2 {
            // use path 00 for ids under 10
            final_path.push("00")
        } else {
            // will not panic, because of prior check if string is more than two long
            final_path.push(id_string[..2].to_owned());
        };
        return Ok(final_path);
    }
}

impl Itemstore {
    // item funcs
    pub async fn get_item(&self, id: MizeId) -> MizeResult<Item> {
        trace!("[ {} ] Itemstore::get_item()", "CALL".yellow());
        trace!("[ {} ] id: {}", "ARG".yellow(), id);

        match self {
            Itemstore::Folder(val) => val.get_item(id).await,
            Itemstore::Memory(val) => val.get_item(id).await,
        }
    }

    pub async fn set_item(&mut self, item: Item) -> MizeResult<()> {
        trace!("[ {} ] Itemstore::set_item()", "CALL".yellow());
        trace!("[ {} ] item: {:?}", "ARG".yellow(), item);

        match self {
            Itemstore::Folder(val) => val.set_item(item).await,
            Itemstore::Memory(val) => val.set_item(item).await,
        }
    }

    pub async fn create_item(&mut self) -> MizeResult<MizeId> {
        trace!("[ {} ] Itemstore::create_item()", "CALL".yellow());

        match self {
            Itemstore::Folder(val) => val.create_item().await,
            Itemstore::Memory(val) => val.create_item().await,
        }
    }

    //pub fn update_item(self) -> MizeResult<I>;

    // other funcs
    pub async fn close(&self) -> MizeResult<()> {
        trace!("[ {} ] Itemstore::close()", "CALL".yellow());
        match self {
            Itemstore::Folder(val) => val.close().await,
            Itemstore::Memory(val) => val.close().await,
        }
    }

    pub async fn next_id(&self) -> u64 {
        trace!("[ {} ] Itemstore::next_id()", "CALL".yellow());

        match self {
            Itemstore::Folder(val) => val.next_id().await,
            Itemstore::Memory(val) => val.next_id().await,
        }
    }
    pub async fn has_item(&self, id: MizeId) -> MizeResult<bool> {
        trace!("[ {} ] Itemstore::has_item()", "CAL".yellow());
        trace!("[ {} ] id: {}", "ARG".yellow(), id);

        let ret = match self {
            Itemstore::Folder(val) => val.has_item(id).await,
            Itemstore::Memory(val) => val.has_item(id).await,
        };

        if let Ok(val) = ret {trace!("[ {} ] Itemstore::has_item(): {}", "RET".yellow(), val)};

        return ret;
    }
}

