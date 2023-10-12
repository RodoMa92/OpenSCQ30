use gtk::{
    glib::{self, Object, Sender},
    subclass::prelude::ObjectSubclassIsExt,
};

use crate::{actions::Action, objects::NamedQuickPreset};

glib::wrapper! {
    pub struct QuickPresetsListing(ObjectSubclass<imp::QuickPresetsListing>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl QuickPresetsListing {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_sender(&self, sender: Sender<Action>) {
        self.imp().set_sender(sender);
    }

    pub fn set_quick_presets(&self, quick_presets: Vec<NamedQuickPreset>) {
        self.imp().set_quick_presets(quick_presets)
    }
}

mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::prelude::*;
    use gtk::{
        glib::{self, clone, subclass::Signal, Sender},
        subclass::{
            prelude::*,
            widget::{CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetImpl},
        },
        template_callbacks, CompositeTemplate,
    };
    use once_cell::sync::Lazy;

    use crate::{actions::Action, objects::NamedQuickPreset, settings::QuickPreset};

    #[derive(Default, CompositeTemplate)]
    #[template(
        resource = "/com/oppzippy/OpenSCQ30/ui/widgets/quick_presets/quick_presets_listing.ui"
    )]
    pub struct QuickPresetsListing {
        #[template_child]
        quick_presets_list: TemplateChild<adw::PreferencesGroup>,

        sender: OnceCell<Sender<Action>>,
        rows: RefCell<Vec<adw::ActionRow>>,
    }

    #[template_callbacks]
    impl QuickPresetsListing {
        #[template_callback]
        pub fn handle_create_clicked(&self, _button: &gtk::Button) {
            let obj = self.obj();

            let root = obj.root().unwrap();
            let window: &gtk::Window = root.downcast_ref().unwrap();

            let dialog = adw::MessageDialog::new(Some(window), Some("Create Custom Profile"), None);
            dialog.add_responses(&[("cancel", "Cancel"), ("create", "Create")]);
            dialog.set_default_response(Some("create"));
            dialog.set_close_response("cancel");
            dialog.set_response_enabled("create", false);
            dialog.set_response_appearance("cancel", adw::ResponseAppearance::Destructive);

            let entry = gtk::Entry::builder()
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .placeholder_text("Name")
                .activates_default(true)
                .build();
            dialog.set_extra_child(Some(&entry));

            entry.connect_changed(clone!(@weak dialog => move |entry| {
                let is_empty = entry.text().trim().is_empty();
                dialog.set_response_enabled("create", !is_empty);
            }));

            dialog.choose(
                gtk::gio::Cancellable::NONE,
                clone!(@weak self as this, @weak entry => move |response| {
                    if response != "create" {
                        return;
                    }
                    this.sender.get()
                        .unwrap()
                        .send(Action::CreateQuickPreset(NamedQuickPreset {
                            name: entry.text().as_str().into(),
                            quick_preset: QuickPreset::default(),
                        }))
                        .unwrap();
                }),
            );
        }
    }

    impl QuickPresetsListing {
        pub fn set_sender(&self, sender: Sender<Action>) {
            self.sender.set(sender.clone()).unwrap();
        }

        pub fn set_quick_presets(&self, mut named_quick_presets: Vec<NamedQuickPreset>) {
            let rows = self.rows.take();
            rows.into_iter()
                .for_each(|item| self.quick_presets_list.remove(&item));

            named_quick_presets
                .sort_by_cached_key(|named_quick_preset| named_quick_preset.name.to_lowercase());
            let rows = named_quick_presets
                .iter()
                .map(|named_quick_preset| {
                    let row = adw::ActionRow::new();
                    row.set_title(&named_quick_preset.name);
                    row.set_activatable(true);
                    row.connect_activated(
                        clone!(@weak self as this, @to-owned named_quick_preset => move |_| {
                            this.activate_quick_preset(&named_quick_preset);
                        }),
                    );

                    let edit_button = gtk::Button::new();
                    edit_button.set_icon_name("document-edit");
                    edit_button.connect_clicked(
                        clone!(@weak self as this, @to-owned named_quick_preset => move |_| {
                            this.edit_quick_preset(&named_quick_preset);
                        }),
                    );
                    row.add_suffix(&edit_button);

                    let delete_button = gtk::Button::new();
                    delete_button.set_icon_name("edit-delete");
                    delete_button.connect_clicked(
                        clone!(@weak self as this, @to-owned named_quick_preset => move |_| {
                            this.delete_quick_preset(&named_quick_preset);
                        }),
                    );
                    row.add_suffix(&delete_button);

                    row
                })
                .collect::<Vec<_>>();

            rows.iter().for_each(|row| self.quick_presets_list.add(row));

            *self.rows.borrow_mut() = rows;
        }

        fn edit_quick_preset(&self, quick_preset: &NamedQuickPreset) {
            self.obj()
                .emit_by_name::<()>("edit-quick-preset", &[quick_preset]);
        }

        fn delete_quick_preset(&self, quick_preset: &NamedQuickPreset) {
            self.sender
                .get()
                .unwrap()
                .send(Action::DeleteQuickPreset(quick_preset.name.to_owned()))
                .unwrap();
        }

        fn activate_quick_preset(&self, quick_preset: &NamedQuickPreset) {
            self.sender
                .get()
                .unwrap()
                .send(Action::ActivateQuickPreset(quick_preset.to_owned()))
                .unwrap();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QuickPresetsListing {
        const NAME: &'static str = "OpenSCQ30QuickPresetsListing";
        type Type = super::QuickPresetsListing;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for QuickPresetsListing {
        fn constructed(&self) {}

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("edit-quick-preset")
                        .param_types([NamedQuickPreset::static_type()])
                        .build(),
                    Signal::builder("activate-quick-preset")
                        .param_types([NamedQuickPreset::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for QuickPresetsListing {}
    impl BoxImpl for QuickPresetsListing {}
}
