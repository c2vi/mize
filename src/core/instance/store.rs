use crate::error::MizeResult;
use crate::item::{Item, ItemData};
use crate::id::MizeId;

pub trait Store {
    // should have the ability to be multi threaded, if the underlying implementation supports 
    // multithreaded IO operations
    // for now all refs to a store hold a Mutex but this mutex is part of where the MizeStore trait
    // is implemented, so that there can be a multithreaded implementation
    //fn get(self, id: MizeId) -> MizeResult<Item<Self>> where Self: Sized;

    fn set<T: Into<ItemData>>(self, id: &MizeId, data: T) -> MizeResult<()>;

    // in the future should implement transactions, ....

    // funcs to do with links, backlinks
    fn get_links(self, item: Item<Self>) -> MizeResult<Vec<MizeId>> where Self: Sized;

    fn get_backlinks(self, item: Item<Self>) -> MizeResult<Vec<MizeId>> where Self: Sized;

    fn new_id(&self) -> MizeResult<MizeId>;

    fn get_value_raw(&self, id: &MizeId) -> MizeResult<Vec<u8>>;
}

/// references an item that is stored in a store
pub trait StoreRef {
    // do we need this or is this basically an item
}
