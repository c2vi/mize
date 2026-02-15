use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::Arc;

use crate::{implementors::{html::HtmlPresenter}, presenter};
use crate::slot::{Slot, SlotTrait};
use crate::presenter::Presenter;
use tracing::info;
use comandr::Comandr;
use flume::{ Receiver, Sender };
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use core::ptr::NonNull;


#[cfg(feature = "wasm-target")]
use wasm_bindgen::JsValue;
#[cfg(feature = "wasm-target")]
use web_sys::js_sys::Function;

//#[cfg(feature = "os-target")]
//use crate::implementors::qt_widget::QtWidgetSlot;
#[cfg(feature = "qt")]
use qt_core::{qs, QString, QTimer, SlotNoArgs};
#[cfg(feature = "qt")]
use qt_widgets::{QApplication, QGridLayout, QWidget};
#[cfg(feature = "qt")]
use qt_gui::{cpp_core::CppBox};


use mize::Module;
use mize::MizeResult;
use mize::error::MizeResultTrait;
use mize::Instance;
use mize::mize_err;
use mize::MizeError;
use mize::proto::MizeMessage;

#[derive(Clone)]
pub struct Mme {
    pub comandr: Arc<Mutex<Comandr>>,
    pub mize: Instance,
}



// console_log macro
// that can be copied into other files for debugging purposes
#[cfg(feature = "wasm-target")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm-target")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
#[cfg(feature = "wasm-target")]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string())})
}





#[no_mangle]
extern "C" fn get_mize_module_mme(empty_module: &mut Box<dyn Module + Send + Sync>, mize: Instance) -> () {
    #[cfg(feature = "wasm-target")]
    console_log!("hiiiiiiiiiiiiiiii from inside get_mize_module_mme!!!!!!!!!");
    let comandr = Comandr::new();
    let new_box: Box<dyn Module + Send + Sync> = Box::new( Mme { comandr: Arc::new(Mutex::new(comandr)), mize, } );

    *empty_module = new_box
}

#[cfg(feature = "wasm-target")]
use mize::platform::wasm::JsInstance;

#[cfg(feature = "wasm-target")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm-target")]
#[wasm_bindgen]
pub unsafe fn wasm_get_mize_module_mme(mut empty_module_ptr: usize, mut mize_ptr: usize) -> () {

    console_log!("hiiiiiiiiiiiiiiiiiiiiiii form wasm_get_mize_module_mme");
    let empty_module = empty_module_ptr as * mut Box<dyn Module + Send + Sync>;
    console_log!("here");

    let mize = (*(mize_ptr as * mut Instance)).clone();

    console_log!("here2");

    //(*empty_module).init(&mize);

    console_log!("here3");

    let comandr = Comandr::new();
    let mut new_box: Box<dyn Module + Send + Sync> = Box::new( Mme { comandr: Arc::new(Mutex::new(comandr)), mize: mize.clone(), } );

    console_log!("before new_box.init()");
    new_box.init(&mize);

    console_log!("before assignment");

    *empty_module = new_box
}
/*
pub fn wasm_get_mize_module_mme(empty_module_ptr: usize, mize: JsInstance) -> usize {
    let mut empty_module: Box<dyn Module + Send + Sync> = *(empty_module_ptr as Box<Box<dyn Module + Send + Sync>>);
    let mize = mize.inner().clone();
    let comandr = Comandr::new();
    let new_box: Box<dyn Module + Send + Sync> = Box::new( Mme { comandr: Arc::new(Mutex::new(comandr)), mize, } );

    *empty_module = new_box
}
*/

impl Module for Mme {
    fn init(&mut self, _instance: &Instance) -> MizeResult<()> {


    #[cfg(feature = "wasm-target")]
    {
    // console log
    use wasm_bindgen::prelude::*;
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
    console_log!("mme module inittttttttttttttttttttttt");
    }

        println!("MmeModule init");

        #[cfg(feature = "os-target")]
        {
            //self.mize.spawn("mme-main", || self.create_x_window());
            let mut cloned_self = self.clone();
            // "significant cross-platform compatibility hazard." xD
            //self.mize.spawn("mme-main", move || cloned_self.create_x_window());
            
            self.create_x_window();

        }

        #[cfg(feature = "wasm-target")]
        {
            use web_sys::js_sys::eval;
            use crate::implementors::html::wasm::MmeJs;
            use std::ptr::NonNull;

            // create mme module object
            let mut mme_js = MmeJs {
                inner: NonNull::from(Box::leak(Box::new(self.clone()))),
                webview_con_id: 0,
            };

            let func = Function::new_with_args("mme_js", r#"
                window.mize.mod.mme = mme_js;
            "#);
            func.call1(&JsValue::null(), &JsValue::from(mme_js)).map_err(|e| mize_err!("from js error: {:?}", e));

            // 
        }

        Ok(())
    }

    fn exit(&mut self, _instance: &Instance) -> MizeResult<()> {
        info!("mme module exit");
        Ok(())
    }

    fn clone_module(&self) -> Box<dyn Module + Send + Sync> {
        Box::new(self.clone())
    }
    
}

#[cfg(feature = "qt")]
unsafe fn qstring_to_rust(q_string: CppBox<QString>) -> String {
    let mut rust_string = String::new();
    let q_string_size = q_string.size();

    for j in 0..q_string_size {
        let q_char = q_string.index_int(j);
        let rust_char = char::from_u32(q_char.unicode() as u32);
        rust_string.push(rust_char.unwrap());
    }
    return rust_string;
}

impl Mme {
    pub fn new(mize: Instance) -> MizeResult<Mme> {
        let comandr = Comandr::new();
        Ok(Mme { comandr: Arc::new(Mutex::new(comandr)), mize, })
    }

    #[cfg(feature = "wasm-target")]
    pub fn create_html_slot() -> MizeResult<()> {
        println!("hi wasm");
        Ok(())
    }

    #[cfg(feature = "os-target")]
    pub fn create_x_window(&mut self) -> MizeResult<()> {
        use std::fs;

        use tao::{
            event::{Event, WindowEvent},
            event_loop::{self, ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
            window::WindowBuilder,
        };
        use wry::{http::{Method, Request, Response}, WebViewBuilder};

        #[cfg(target_os = "linux")]
        use wry::WebViewExtUnix;

        #[cfg(target_os = "linux")]
        use webkit2gtk::{Settings, WebInspectorExt};

        #[cfg(target_os = "linux")]
        use webkit2gtk::WebViewExt;

        use crate::implementors::html::webview_con::{msg_from_string, msg_to_string};

        let event_loop = EventLoopBuilder::with_user_event().build();
        let event_loop_proxy: EventLoopProxy<MizeMessage> = event_loop.create_proxy();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))]
        let builder = WebViewBuilder::new(&window);

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        )))]
        let builder = {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            let vbox = window.default_vbox().unwrap();
            WebViewBuilder::new_gtk(vbox)
        };

        //let html_str = fs::read_to_string(format!("{}/../implementors/html/js-runtime/dist/index.html", file!()))?;
        //println!("html_str: {}", html_str);

        // get the path of the index.html
        let test = self.mize.fetch_module("mme")?;
        let mme_module_path = self.mize.fetch_module("cross.wasm32-none-unknown.mme")?;

        let mize_module_path = self.mize.fetch_module("cross.wasm32-none-unknown.mize")?;

        let init_script = format!(r#"
            import("{mize_module_path}/mize.js").then( module => module.init_mize({{module_dir: {{mize: "{mize_module_path}"}}}}))
        "#);


        // add the mize connection to the instance inside the webview
        let (tx, rx): (Sender<MizeMessage>, Receiver<MizeMessage>) = flume::unbounded();
        let conn_id = self.mize.new_connection(tx)?;


        let mut self_clone = self.clone();
        let mut self_clone_two = self.clone();
        let webview = builder
        //.with_url("http://localhost:8000/index.html")
        //.with_url("file:/
            //home/me/tmp/rd/test.html")
        //.with_url(format!("file://{}/index.html", mme_module_path))
        //.with_html("hello worldddddddddddddddddddddddddddddddddd".to_owned())
        //.with_initialization_script(init_script.as_str())
        .with_custom_protocol(
          "wry".into(),
          move | request| {
            
            // the body will be a mize message as a string
            if request.method() == &Method::POST {
                let msg_str = request.headers().get("MizeMsg").unwrap().to_str().unwrap();

                println!("webview_con incoming got msg: {}", msg_str);

                let msg = match msg_from_string(msg_str.to_owned(), conn_id) {
                    Ok(val) => val,
                    Err(err) => {
                        self_clone_two.mize.report_err(err.into());
                        return http::Response::builder()
                            .header(CONTENT_TYPE, "text/plain")
                            .status(500)
                            .body("".to_string().as_bytes().to_vec())
                            .unwrap()
                            .map(Into::into);
                    },
                };

                self_clone_two.mize.got_msg(msg);


                return http::Response::builder()
                    .header(CONTENT_TYPE, "text/plain")
                    .status(http::StatusCode::OK)
                    .body("".to_string().as_bytes().to_vec())
                    .unwrap()
                    .map(Into::into);
            }

            match get_wry_response(request, self_clone_two.clone()) {
                Ok(r) => r.map(Into::into),
                Err(e) => {
                    println!("get_wry_response error: {}", e);
                    http::Response::builder()
                        .header(CONTENT_TYPE, "text/plain")
                        .status(500)
                        .body(e.to_string().as_bytes().to_vec())
                        .unwrap()
                        .map(Into::into)
                },
            }
          }
        )
        // tell the webview to load the custom protocol
        .with_url("wry://localhost")
        //.with_ipc_handler(move | res: Request<String> | {
            //crate::implementors::html::webview_con::ipc_handler(res, self_clone.clone(), conn_id)
        //})
        //.with_html(html_str)
        /*
        .with_drag_drop_handler(|e| {
          match e {
            wry::DragDropEvent::Enter { paths, position } => {
              println!("DragEnter: {position:?} {paths:?} ")
            }
            wry::DragDropEvent::Over { position } => println!("DragOver: {position:?} "),
            wry::DragDropEvent::Drop { paths, position } => {
              println!("DragDrop: {position:?} {paths:?} ")
            }
            wry::DragDropEvent::Leave => println!("DragLeave"),
            _ => {}
          }

          true
        })
        */
        .build()?;
        //_webview.open_devtools();
        println!("{}", webview.url().unwrap());
        let req = Request::builder()
              .uri(webview.url().unwrap().to_string())
              .body("hooooooooooooo")
              .unwrap();

        #[cfg(target_os = "linux")]
        {
            let settings = Settings::builder()
                .allow_file_access_from_file_urls(true)
                .enable_developer_extras(true)
                .build();
            let __webview = webview.webview();
           __webview.set_settings(&settings);

            let inspector = __webview.inspector().expect("no inspector");
            inspector.show();
        }


        crate::implementors::html::webview_con::mme_setup_weview_con_host(self, rx, event_loop_proxy)?;
        let cloned_self = self.clone();


        // this is where we block the main thread....
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit
                },

                Event::UserEvent::<MizeMessage>(msg) => {
                    match msg_to_string(msg) {
                        Ok(msg_string) => {
                            webview.evaluate_script(format!("mize.mod.mme.webview_msg_recv_fn({})", msg_string).as_str());
                        },
                        Err(err) => {
                            cloned_self.mize.report_err(err.into());
                        },
                    };
                },

                _ => {},
            }

        });
    }


    #[cfg(features = "qt")]
    pub fn create_qt_slot(&self) -> MizeResult<()> {
        unsafe {

            let backend = i_slint_backend_qt::Backend::new();


            let main_widget = qt_widgets::QWidget::new_0a();
            main_widget.show();
            main_widget.set_window_title(&qs("mme_main"));



            comandr::platform::qt::init(main_widget.as_ptr());
          
            //let other_window = OtherWindow::new().unwrap();
            //other_window.show();

            let presenter: Presenter = Presenter::HtmlPresenter(HtmlPresenter::from_folder(Path::new("/home/me/work/mme-presenters/presenters/hello-world").to_owned())?);

            let mut slot: Slot = Slot::QtWidgetSlot(QtWidgetSlot::from_widget(main_widget)?);

            slot.load(presenter);


            println!("run_event_loop");
            unsafe {
                qt_core::QCoreApplication::exec();
            }
            //backend.run_event_loop();
            //unsafe {
                //run_my_event_loop(my_app);
            //}

            Ok(())
        }
    }
}


#[cfg(feature = "os-target")]
use tao::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};
#[cfg(feature = "os-target")]
use wry::{
  http::{self, header::CONTENT_TYPE, Request, Response},
  WebViewBuilder,
};

#[cfg(feature = "os-target")]
fn get_wry_response(request: Request<Vec<u8>>, mut mme: Mme) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
    use comandr::core::module;
    use mize::item::ItemData;
    use mize::error::MizeResultTrait;




    let mut path = PathBuf::from(request.uri().path());

    let root = if path.starts_with("/modules") {
        let mod_name = path.iter().nth(2).unwrap().to_str().unwrap();
        println!("get_wry_response: modname: {}", mod_name);
        let mod_dir = PathBuf::from(mme.mize.fetch_module(format!("cross.wasm32-none-unknown.{}", mod_name).as_str()).unwrap());
        mod_dir
    } else {
        PathBuf::from(mme.mize.fetch_module("cross.wasm32-none-unknown.mme").unwrap())
    };

    let mut module_dir_conf: ItemData = mme.mize.get("self/config/module_dir/cross/wasm32-none-unknown").as_std()?.as_data_full().as_std()?;
    let mut module_dir_str = String::new();
    if !module_dir_conf.cbor().is_null() {
        let map = module_dir_conf.cbor().as_map().ok_or(mize_err!("self/config/module_dir was not a map")).as_std()?;
        for (mod_name, module_dir) in map {
            let mod_name_str = mod_name.as_text().ok_or(mize_err!("mod_name was not a string")).as_std()?;
            module_dir_str += format!(r#" "{mod_name_str}": "wry://localhost/modules/{mod_name_str}", "#).as_str();
        }
    }


    let index_html = format!(r#"
        <html>
          <head>
            <script>
                import("/modules/mize/mize.js").then( module => module.init_mize({{
                    module_dir: {{ {module_dir_str} }},
                    load_modules: "mme",
                    module_url: "wry://localhost/modules"
                }}))
            </script>
          </head>
          <body>
            MME loading...
          </body>  
        </html>
    "#);

    let path = if path == PathBuf::from("/") {
        // return the index.html
        return Response::builder()
            .header(CONTENT_TYPE, "text/html")
            .body(index_html.into_bytes())
            .map_err(Into::into);

    } else if path.starts_with("/modules") {
        path.iter().skip(3).collect()
    } else {
        //  removing leading slash
        let mut string = path.into_os_string().into_string().unwrap();
        string.remove(0);
        PathBuf::from(string)
    };

    println!("root: {}", root.display());
    println!("path: {}", path.display());
    let content = std::fs::read(std::fs::canonicalize(root.join(path.as_path()))?)?;
    let path_str = path.as_os_str().to_str().unwrap();

    // Return asset contents and mime types based on file extentions
    // If you don't want to do this manually, there are some crates for you.
    // Such as `infer` and `mime_guess`.
    let mimetype = if path_str.ends_with(".html") || path == PathBuf::from("/") {
        "text/html"
    } else if path_str.ends_with(".js") {
        "text/javascript"
    } else if path_str.ends_with(".png") {
        "image/png"
    } else if path_str.ends_with(".wasm") {
        "application/wasm"
    } else {
        "text/html"
    };
    println!("mimetype: {}", mimetype);

    Response::builder()
        .header(CONTENT_TYPE, mimetype)
        .body(content)
        .map_err(Into::into)
}


