use enum_dispatch::enum_dispatch;

#[cfg(features = "slint")]
use crate::implementors::slint_widget::SlintWidget;

#[cfg(features = "qt")]
use crate::implementors::qt_widget::QtWidgetSlot;

use crate::implementors::html::HtmlSlot;

use crate::presenter::Presenter;
use mize::MizeResult;

// common behaviour for all SlotTypes
#[enum_dispatch]
pub trait SlotTrait {
    fn load(&mut self, presenter: Presenter) -> MizeResult<()>;

    //pub fn load_html(pressenter: HtmlPresenter) -> MizeResult<()> {}
}

// this is the main type of this project
// a Widget represents any kind of "screen realestate"
// be it a Webview, slint widget, Xwindow, ...

// an enum over all of what types such "screen realestate" could have
#[enum_dispatch(SlotTrait)]
pub enum Slot {
    //Xwindow {},
    //WaylandWindow {},
    //SlintWidget,
    #[cfg(features = "qt")]
    QtWidgetSlot,
    //GtkWidget {},
    HtmlSlot,
    //QuarzWindow {},
    //NtWindow {},
    //Activity {},
}


pub struct Position {
    x: u32,
    y: u32,
    z: u32,
}


