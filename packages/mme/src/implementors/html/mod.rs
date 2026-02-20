
use std::path::PathBuf;
use comandr::Comandr;

use crate::slot::SlotTrait;
use crate::mme::Mme;
use mize::MizeResult;

pub mod webview_con;


pub struct HtmlPresenter {
    pub path: PathBuf,
}

impl HtmlPresenter {
    pub fn from_folder(path: PathBuf) -> MizeResult<HtmlPresenter> {
        Ok(HtmlPresenter { path })
    }
    
}

pub struct HtmlSlot {
}

impl SlotTrait for HtmlSlot {
    fn load(&mut self,presenter:crate::presenter::Presenter) -> MizeResult<()> {
        Ok(())
    }
}


#[cfg(feature = "wasm-target")]
pub mod wasm {

    use comandr::Comandr;
    use comandr::Command;
    use web_sys::console;
    use web_sys::EventTarget;
    use web_sys::js_sys::Function;
    use std::panic;
    use std::ptr::NonNull;
    use std::slice::Iter;
    use comandr::Module;
    use comandr::ComandrResult;

    use crate::mme::Mme;

    // console log
    use wasm_bindgen::prelude::*;

    use super::webview_con::msg_from_string;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }
    macro_rules! console_log {
        // Note that this is using the `log` function imported above during
        // `bare_bones`
        ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
    }
    // end of console log

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    #[wasm_bindgen]
    pub struct MmeJs {
        pub inner: NonNull<Mme>,
        pub webview_con_id: u64,
    }

    #[wasm_bindgen]
    impl MmeJs {

        #[wasm_bindgen]
        pub unsafe fn webview_msg_recv_fn(&mut self, msg_string: String) -> () {

            let mize = &self.inner.as_mut().mize;

            let msg = match msg_from_string(msg_string, self.webview_con_id) {
                Ok(val) => val,
                Err(err) => {
                    mize.report_err(err.into());
                    return;
                },
            };

            mize.got_msg(msg);
        }

        #[wasm_bindgen]
        pub unsafe fn comandr_search(&mut self, string: String) -> Vec<String> {
            //self.inner.as_mut().comandr.search(string)
            Vec::new()
        }

        pub unsafe fn comandr_list(&mut self) -> Vec<String> {
            //self.inner.as_mut().comandr.list_commands()
            Vec::new()
        }

        #[wasm_bindgen]
        pub unsafe fn comandr_run(&mut self, name: String, args: Vec<String>) -> () {
            //self.inner.as_mut().comandr.execute(name, args)

        }

    }


    pub struct MmeComandrModule {
        commands: Vec<Command>
    }

    fn reload_page() -> ComandrResult<()>{
        let js_fn_str = r#"
            window.location.reload()
        "#;
        let js_fn = Function::new_no_args(js_fn_str);
    
        js_fn.call0(&web_sys::wasm_bindgen::JsValue::NULL);

        Ok(())
    }

    impl MmeComandrModule {
        pub fn new() -> MmeComandrModule {
            let commands = vec![
                Command { name: "reload".to_owned(), closure: Box::new(reload_page) },
            ];
            MmeComandrModule { commands }
        }
    }

    impl Module for MmeComandrModule {
        fn name(&self) -> String {
            "mme".to_owned()
        }

        fn commands(&self) -> Iter<'_, Command> {
            self.commands.iter()
        }

        fn get_command(&mut self, name: String) -> Option<&mut Command> {
            for command in self.commands.iter_mut() {
                if command.name == name {
                    return Some(command);
                }
            }
            return None;
        }
    }

}



