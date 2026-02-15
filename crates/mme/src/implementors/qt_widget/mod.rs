use qt_gui::cpp_core::Ptr;
use tracing::{error, debug};
use qt_core::{qs, QBox, QPointerOfQObject};
use qt_widgets::QWidget;
use qt_widgets::cpp_core::CppBox;
use cpp::cpp;
use std::ffi::{CStr, CString};

use crate::presenter::Presenter;
use crate::slot::{Slot, SlotTrait};
use mize::MizeResult;


pub struct HtmlAdapter {
}

impl HtmlAdapter {
    pub fn sub_slot(&self) -> MizeResult<Slot> {
        todo!()
    }
}

pub struct QtWidgetSlot {
    widget: QBox<QWidget>,
}

impl QtWidgetSlot {
    pub fn from_widget(widget: QBox<QWidget>) -> MizeResult<QtWidgetSlot> {
        Ok(QtWidgetSlot { widget })
    }
}

cpp! {{
    #include <stdint.h>
    #include <stdio.h>
    #include <cstdio>
    #include <QtWidgets/QApplication>
    #include <QtWidgets/QMainWindow>
    #include <QtWidgets/QPushButton>
    #include <QtWidgets/QGridLayout>
    #include <QtCore/QPointer>
    #include <QtCore/QDebug>
    #include <QtCore/QTimer>

//#include <QtWebEngine/QtWebEngine>
#include <QtWebEngineWidgets/QWebEngineView>
#include <QtCore/QUrl>

}}

impl SlotTrait for QtWidgetSlot {
    fn load(&mut self, presenter: Presenter) -> MizeResult<()> {
        match presenter {
            Presenter::HtmlPresenter(html) => {
                // here would go the code, to load the index.html of the HtmlPresenter's path into
                // the self.widget....
                println!("loading html into widget....");
                let url = format!("file://{}/index.html", html.path.display());

                let c_string: CString = CString::new(url.as_str()).unwrap();
                let c_str: &CStr = c_string.as_c_str();

                unsafe {
                    create_webview(self.widget.as_ptr(), c_str.as_ptr());

                    self.widget.set_style_sheet(&qs("border: 2px solid blue;"));
                    

                    //let widget_full = QWidget::new_0a();
                    //let widget = widget_full.as_raw_ptr();
                    //
                    //widget_fill(&mut self.widget);

                    //cpp_widget_fill(&mut self.widget);


                }
            }
        }

        Ok(())
    }
}

extern "C" { fn create_webview(widget: Ptr<QWidget>, url: * const i8); }

pub unsafe fn widget_fill(widget: &mut QBox<QWidget>) {
    let grid_layout = unsafe { qt_widgets::QGridLayout::new_0a() };
    widget.set_layout(&grid_layout);

    let button = unsafe { qt_widgets::QPushButton::new() };
    button.set_text(&qs("hello from rust button"));

    let button2 = unsafe { qt_widgets::QPushButton::new() };
    button2.set_text(&qs("hello from another button"));
    
    grid_layout.add_widget_3a(&button, 0, 1);
    grid_layout.add_widget_3a(&button2, 1, 0);

}

/*
pub unsafe fn cpp_widget_fill(widget: &mut QBox<QWidget>) {
    let a: i32 = 5;
    let widget = widget.as_raw_ptr();

    let x: i32 = cpp!(unsafe [widget as "QWidget*", a as "int32_t"] -> i32 as "int32_t" {

        printf("hello from macro\n");

        //QWidget * mywidget = new QWidget();

        //QUrl url;
        //url.setScheme("http");
        //url.setHost("www.orf.at");

        //QWebEngineView view = QWebEngineView(widget);
        //view.setUrl(url);
        //view.resize(1024, 750);
        //view.show();

        /*
        QGridLayout *layout = new QGridLayout(widget);
        widget->setLayout(layout);

        QPushButton * button = new QPushButton(widget);
        button->setText("My text");
        button->setToolTip("A tooltip");

        QPushButton * button2 = new QPushButton(widget);
        button2->setText("My text 2");
        button2->setToolTip("A tooltip 2");

        QPushButton * button3 = new QPushButton(widget);
        button3->setText("My text 3");
        button3->setToolTip("A tooltip 3");

        layout->addWidget (button, 0, 0);
        layout->addWidget (button2, 0, 1);
        layout->addWidget (button3, 1, 1);
        */

        widget->show();

        return a;
    });
}
*/




