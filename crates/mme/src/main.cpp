
#include <stdint.h>
#include <stdio.h>
#include <iostream>

#include <QtWidgets/QApplication>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QGridLayout>
#include <QtCore/QPointer>
#include <QtCore/QDebug>
#include <QtCore/QTimer>
#include <QtCore/QString>

//#include <QtWebEngine/QtWebEngine>
#include <QtWebEngineWidgets/QWebEngineView>
#include <QtCore/QUrl>

extern "C" void run_my_event_loop( QPointer<QApplication> my_app );
extern "C" void hello_two();
extern "C" void hello_three();
extern "C" QPointer<QWidget> get_stuff();

extern "C" void create_webview(QWidget *widget, char * url_raw) {
      
  printf("in create_webview\n");

  QString * url_string = new QString(url_raw);
  QUrl url = QUrl::fromUserInput(*url_string);

  QWebEngineView * view = new QWebEngineView(widget);
  view->setUrl(url);
  //view->resize(1024, 750);

  QGridLayout *layout = new QGridLayout(widget);
  layout->setContentsMargins(0, 0, 0, 0);
  widget->layout()->setContentsMargins(0, 0, 0, 0);
  QWidget * view_widget = (QWidget*) view;
  view_widget->layout()->setContentsMargins(0, 0, 0, 0);

  layout->addWidget ((QWidget*)view, 0, 0);

  return;


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
  layout->addWidget ((QWidget*)view, 1, 0);
}

/*
class Foo : public QObject
{
    Q_OBJECT

    public:
    Foo( QObject* parent = 0 ) : QObject( parent )
    {}

    private:
    void doStuff()
    {
        qDebug() << "Emit signal one";
        emit signal1();

        qDebug() << "Emit finished";
        emit finished();

        qDebug() << "Emit signal two";
        emit signal2();
    }

    signals:
    void signal1();
    void signal2();

    void finished();

    public slots:
    void slot1()
    {
        qDebug() << "Execute slot one";
    }

    void slot2()
    {
        qDebug() << "Execute slot two";
    }

    void start()
    {
        printf("in start of foo");
        //doStuff();

        qDebug() << "Bye!";
    }
};
*/

//extern "C" void place_slint_widget(QWidget * slint_widget) {
  //printf("hello from test\n");
//}

extern "C" QPointer<QApplication> init_app() {
  int argc = 0;
  char hi = 'c';
  char * two = &hi;

  QPointer<QApplication> my_app = new QApplication (argc, &two);


  QPointer<QWidget> main_widget = get_stuff();

  QMainWindow * my_window = new QMainWindow();
  my_window->setCentralWidget(main_widget);
  //my_window->show();

  //my_app->exec();

  //run_my_event_loop(my_app);

  return my_app;
}

extern "C" QPointer<QWidget> get_stuff() {
  QPointer<QWidget> centralWidget = new QWidget();

  QGridLayout *layout = new QGridLayout();
  centralWidget->setLayout(layout);

  QPushButton button;
  button.setText("My text");
  button.setToolTip("A tooltip");

  QPushButton button2;
  button2.setText("My text 2");
  button2.setToolTip("A tooltip 2");

  QPushButton button3;
  button3.setText("My text 3");
  button3.setToolTip("A tooltip 3");

  layout->addWidget (&button, 0, 0);
  layout->addWidget (&button2, 0, 1);
  layout->addWidget (&button3, 1, 1);

  return centralWidget;
}

extern "C" void hello_three() {

  QWidget *centralWidget = new QWidget();

  QGridLayout *layout = new QGridLayout();
  centralWidget->setLayout(layout);

  QPushButton button;
  button.setText("My text");
  button.setToolTip("A tooltip");

  QPushButton button2;
  button2.setText("My text 2");
  button2.setToolTip("A tooltip 2");

  QPushButton button3;
  button3.setText("My text 3");
  button3.setToolTip("A tooltip 3");

  layout->addWidget (&button, 0, 0);
  layout->addWidget (&button2, 0, 1);
  layout->addWidget (&button3, 1, 1);

  centralWidget->show();

}

extern "C" void hello_two() {

  QCoreApplication * app = QApplication::instance();

  QPushButton button;
  button.setText("My text");
  button.setToolTip("A tooltip");

  button.show();

  /*
  Foo foo;

  QObject::connect( &foo, &Foo::signal1, &foo, &Foo::slot1 );
  QObject::connect( &foo, &Foo::signal2, &foo, &Foo::slot2 );

  //QObject::connect( &foo, &Foo::finished, app, &QCoreApplication::quit );

  QTimer::singleShot( 0, &foo, &Foo::start );
  */

}

extern "C" void run_my_event_loop( QPointer<QApplication> my_app ) {

  //QCoreApplication * app = QApplication::instance();

  //QPushButton other_button;
  //other_button.setText("My text");
  //other_button.setToolTip("A tooltip");

  //other_button.show();


  //printf("app exec\n");
  int result = my_app->exec();

  printf("app exited with: %d\n", result);
}

//#include "main.moc"



