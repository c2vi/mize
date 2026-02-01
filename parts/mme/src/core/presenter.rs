use enum_dispatch::enum_dispatch;

use crate::implementors::html::HtmlPresenter;


// A Presenter takes data (in the form of MiZe Items) and presents them in the form of a gui.
// It can be put into a Space and provide sub Spaces, where other Presenters can be put into.
// It can provide different implementations for the different types of spaces
#[enum_dispatch]
pub trait PresenterTrait {
}

#[enum_dispatch(PresenterTrait)]
pub enum Presenter {
    HtmlPresenter,
}

impl Presenter {
}
