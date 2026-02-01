use crate::slot::Slot;

use mize::MizeResult;

// A Presenter, that takes not data and produces only one output Space
// with that space being of a different type.
// In order to put Presenters of different types into one another.
// eg a QT-Webview is put into a qt-space ... in order to get a web-space....
pub struct Adapter {
}

pub trait AdapterTrait {
    fn sub_slot() -> MizeResult<Slot>;
}
