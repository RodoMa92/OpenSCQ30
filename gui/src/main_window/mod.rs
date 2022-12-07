mod imp;

use gtk::{
    glib::{self, Object},
    subclass::prelude::ObjectSubclassIsExt,
};

use crate::device_selection::Device;

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl MainWindow {
    pub fn new() -> Self {
        Object::new(&[])
    }

    pub fn set_devices(&self, devices: &[Device]) {
        self.imp().set_devices(devices);
    }
}
