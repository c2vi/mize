use std::fs;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use sysinfo::{System, Pid, ProcessRefreshKind, RefreshKind};
use std::io::Write;
use ciborium::Value as CborValue;

use crate::instance::store::IdIter;
use crate::item::IntoItemData;
use crate::memstore::MemStore;
use crate::{core::instance::store::Store, mize_err};
use crate::error::{IntoMizeResult, MizeResult, MizeError};
use crate::core::item::{ItemData, Item};
use crate::core::id::MizeId;
use crate::item::get_raw_from_cbor;
use crate::instance::Instance;

#[derive(Clone, Debug)]
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path_str: &str) -> MizeResult<FileStore> {

        let path = Path::new(path_str);

        // create that path if it does not exist
        fs::create_dir_all(Path::new(&path).join("store"))?;

        // check for valid pid file
        if let Some(pid) = valid_pid_file(path)? {
            // store already opened
            return Err(mize_err!("MizeStore at path {} is already opened by process with pid {}", path.display(), pid));

        } else {
            // write our own pid file
            fs::remove_file(path.join("pid"));
            let pid = std::process::id();
            let mut file = OpenOptions::new().write(true).create(true).open(path.join("pid"))?;
            write!(file, "{}", pid)?;
        }

        // init the store
        fs::write(path.join("next_id"), "1");

        Ok(FileStore { path: Path::new(&path).to_owned() })
    }

    pub fn store_is_opened(store_path: String) -> MizeResult<bool> {
        println!("valid_pid_file: {}", valid_pid_file(Path::new(&store_path))?.is_some());
        Ok(valid_pid_file(Path::new(&store_path))?.is_some())
    }
}

impl Store for FileStore {
    fn new_id(&self) -> MizeResult<String> {

        let mut next_id: u64 = String::from_utf8(fs::read(self.path.join("next_id"))
            .mize_result_msg(format!("could not read next_id at '{}'", self.path.display()))?)?
            .parse()
            .mize_result_msg(format!("could not parse next_id at '{}' to u64", self.path.display()))?;

        println!("nextid fsstore: {}", next_id);
        let id_string = format!("{}", next_id);

        next_id += 1;

        fs::write(self.path.join("next_id"), format!("{}", next_id))?;

        return Ok(id_string);
    }

    fn set(&self, id: MizeId, data: ItemData) -> MizeResult<()> {
        println!("setting: {:?}", id);
        let path = self.path.join("store").join(id.namespace_str()).join(id.store_part());
        println!("path: {:?}", &path);
        fs::create_dir_all(self.path.join("store").join(id.namespace_str()))?;

        let file = OpenOptions::new().write(true).create(true).open(path)?;

        ciborium::into_writer(data.cbor(), file)?;
        Ok(())
    }

    fn get_links(&self, item: Item) -> MizeResult<Vec<MizeId>> {
        Ok(Vec::new())
    }

    fn get_backlinks(&self, item: Item) -> MizeResult<Vec<MizeId>> {
        Ok(Vec::new())
    }

    fn get_value_raw(&self, id: MizeId) -> MizeResult<Vec<u8>> {
        let path = self.path.join("store").join(id.namespace_str()).join(id.store_part());

        let file = OpenOptions::new().read(true).create(true).open(path)?;

        let cbor_val: CborValue = ciborium::from_reader(file)
            .mize_result_msg(format!("could not read file '{}' from FileStore", self.path.join("store").join(id.store_part()).display()))?;

        let path = id.path();
        let mut id_iter = path.iter();
        id_iter.next();
        let id_without_first = id_iter.collect();

        Ok(get_raw_from_cbor(&cbor_val, id_without_first)?.to_owned())
    }

    fn get_value_data_full(&self, id: MizeId) -> MizeResult<ItemData> {
        let file_path = self.path.join("store").join(id.namespace_str()).join(id.store_part());
        println!("file_path: {:?}", file_path.display());
        let file = OpenOptions::new().read(true).open(file_path)?;

        let cbor_value: CborValue = ciborium::from_reader(file)
            .mize_result_msg(format!("could not read file '{}' from FileStore", self.path.join("store").join(id.store_part()).display()))?;
        let data: ItemData = cbor_value.into_item_data();

        let tmp = id.path();
        let mut path_iter = tmp.into_iter();
        path_iter.next();
        let new_path: Vec<String> = path_iter.map(|v| v.to_owned()).collect();
        let ret_data = data.get_path(new_path)?;

        return Ok(ret_data);
    }
    fn id_iter(&self) -> MizeResult<IdIter> {
        todo!()
    }
    fn next_id(&self, prev_id: &str) -> MizeResult<Option<String>> {
        todo!()
    }
    fn first_id(&self) -> MizeResult<String> {
        Ok("0".to_owned())
    }
}

fn valid_pid_file(path: &Path) -> MizeResult<Option<u32>> {

    let pid_file_path = path.join("pid");

    // if the pid file does not exist, there is no pid...
    if !pid_file_path.exists() {
        return Ok(None);
    }

    // read pid file
    let pid: u32 = String::from_utf8(fs::read(&pid_file_path)
        .mize_result_msg(format!("Could not read contents of pid file at '{}'", pid_file_path.display()))?)?
        .parse()
        .mize_result_msg(format!("Could not parse contents of pid file at '{}' to u32", pid_file_path.display()))?;

    let refresh_kind = RefreshKind::new().with_processes(ProcessRefreshKind::everything());
    let system = sysinfo::System::new_with_specifics(refresh_kind);

    if let Some(_) = system.process(Pid::from(pid as usize)) {
        return Ok(Some(pid));
    } else {
        return Ok(None);
    }
}

