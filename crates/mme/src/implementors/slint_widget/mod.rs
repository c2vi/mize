use std::path::{Path, PathBuf};
use qt_gui::q_touch_event::touch_point;
use slint_interpreter::{ComponentInstance, ComponentDefinition, ComponentCompiler, Value, SharedString, ComponentHandle};

use crate::presenter::Presenter;
use crate::slot::SlotTrait;
use crate::slot::Position;
use mize::MizeResult;

//pub mod qt_backend;



pub struct SlintWidget {
    inner: ComponentInstance,
}


impl SlintWidget {
    fn from_slint_file(path: PathBuf) -> MizeResult<SlintWidget> {
        todo!()
    }

    fn sample() -> MizeResult<SlintWidget> {

        let code = r#"
            export component MyWin inherits Window {
                in property <string> my_name;
                Text {
                    text: "Hello, " + my_name;
                }
            }
        "#;

        let mut compiler = ComponentCompiler::default();
        let definition = spin_on::spin_on(compiler.build_from_source(code.into(), Default::default()));
        assert!(compiler.diagnostics().is_empty());
        let instance = definition.unwrap().create().unwrap();
        instance.set_property("my_name", Value::from(SharedString::from("World"))).unwrap();
        return SlintWidget::from_instance(instance);
    }

    fn from_instance(instance: ComponentInstance) -> MizeResult<SlintWidget> {
        Ok(SlintWidget {
            inner: instance,
        })
    }
}

impl SlotTrait for SlintWidget {
    fn load(&mut self,presenter:Presenter) -> MizeResult<()> {
        todo!()
    }
}






