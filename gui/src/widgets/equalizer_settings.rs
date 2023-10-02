use gtk::{
    glib::{self, Object, Sender},
    subclass::prelude::ObjectSubclassIsExt,
};
use openscq30_lib::packets::structures::EqualizerConfiguration;

use crate::{actions::Action, objects::CustomEqualizerProfileObject};

glib::wrapper! {
    pub struct EqualizerSettings(ObjectSubclass<imp::EqualizerSettings>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl EqualizerSettings {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_sender(&self, sender: Sender<Action>) {
        self.imp().set_sender(sender);
    }

    pub fn set_equalizer_configuration(&self, equalizer_configuration: EqualizerConfiguration) {
        self.imp()
            .set_equalizer_configuration(equalizer_configuration);
    }

    pub fn equalizer_configuration(&self) -> EqualizerConfiguration {
        self.imp().equalizer_configuration()
    }

    pub fn set_custom_profiles(&self, custom_profiles: Vec<CustomEqualizerProfileObject>) {
        self.imp().set_custom_profiles(custom_profiles)
    }
}

mod imp {
    use std::cell::{Cell, OnceCell};

    use gtk::{
        gio,
        glib::{self, clone, Sender},
        prelude::*,
        subclass::{
            prelude::*,
            widget::{
                CompositeTemplateCallbacksClass, CompositeTemplateClass, WidgetClassSubclassExt,
                WidgetImpl,
            },
        },
        CompositeTemplate, Expression, PropertyExpression, SignalListItemFactory, TemplateChild,
    };
    use openscq30_lib::packets::structures::{
        EqualizerConfiguration, PresetEqualizerProfile, VolumeAdjustments,
    };
    use strum::IntoEnumIterator;

    use crate::{actions::Action, objects::BoxedVolumeAdjustments, widgets::Equalizer};
    use crate::{
        objects::{CustomEqualizerProfileObject, EqualizerProfileObject},
        widgets::EqualizerProfileDropdownRow,
    };

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/oppzippy/OpenSCQ30/equalizer_settings.ui")]
    pub struct EqualizerSettings {
        #[template_child]
        pub equalizer: TemplateChild<Equalizer>,
        #[template_child]
        pub profile_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub custom_profile_selection: TemplateChild<gtk::Box>,
        #[template_child]
        pub custom_profile_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub create_custom_profile_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_custom_profile_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub custom_profile_buttons: TemplateChild<gtk::Box>,

        profiles: OnceCell<gio::ListStore>,
        custom_profiles: OnceCell<gio::ListStore>,

        custom_profile_index: Cell<Option<u32>>,
        sender: OnceCell<Sender<Action>>,
    }

    #[gtk::template_callbacks]
    impl EqualizerSettings {
        pub fn set_sender(&self, sender: Sender<Action>) {
            self.sender.set(sender.clone()).unwrap();
        }

        #[template_callback]
        fn handle_create_custom_profile(&self, _button: &gtk::Button) {
            self.obj()
                .activate_action(
                    "win.create-custom-equalizer-profile",
                    Some(
                        &BoxedVolumeAdjustments(
                            self.equalizer.volume_adjustments().to_vec().into(),
                        )
                        .to_variant(),
                    ),
                )
                .unwrap();
        }

        #[template_callback]
        fn handle_delete_custom_profile(&self, _button: &gtk::Button) {
            if let Some(profile) = self.custom_profile_dropdown.selected_item() {
                self.sender
                    .get()
                    .unwrap()
                    .send(Action::DeleteCustomEqualizerProfile(
                        profile.downcast().unwrap(),
                    ))
                    .unwrap();
            }
        }

        #[template_callback]
        fn handle_volumes_changed(&self, equalizer: &Equalizer) {
            self.update_custom_profile_selection();
            // apply-equalizer-settings fires instantly when changing the preset profile, so we only need to be concerned
            // with custom profiles here.
            let selected_profile = self.profile_dropdown.selected_item().map(|item| {
                item.downcast::<EqualizerProfileObject>()
                    .expect("item must be EqualizerProfileObject")
            });
            let volume_adjustments_match_preset_profile = selected_profile
                .and_then(|profile_object| {
                    PresetEqualizerProfile::from_id(profile_object.profile_id() as u16)
                })
                .map(|profile| profile.volume_adjustments())
                .map(|volume_adjustments| {
                    equalizer.volume_adjustments() == volume_adjustments.adjustments()
                })
                .unwrap_or(false);
            if !volume_adjustments_match_preset_profile {
                if let Some(custom_profile_index) = self.custom_profile_index.get() {
                    self.profile_dropdown.set_selected(custom_profile_index);
                    self.sender
                        .get()
                        .unwrap()
                        .send(Action::SetEqualizerConfiguration(
                            self.equalizer_configuration(),
                        ))
                        .unwrap();
                }
            }
        }

        pub fn equalizer_configuration(&self) -> EqualizerConfiguration {
            if self.is_custom_profile() {
                EqualizerConfiguration::new_custom_profile(VolumeAdjustments::new(
                    self.equalizer.volume_adjustments(),
                ))
            } else {
                let selection = self
                    .profile_dropdown
                    .selected_item()
                    .expect("an item must be selected")
                    .downcast::<EqualizerProfileObject>()
                    .expect("selected item must be an EqualizerProfileObject");
                EqualizerConfiguration::new_from_preset_profile(
                    PresetEqualizerProfile::from_id(selection.profile_id() as u16).unwrap_or_else(
                        || {
                            panic!(
                                "equalizer preset with selected profile id {} not found",
                                selection.profile_id()
                            );
                        },
                    ),
                )
            }
        }

        pub fn set_equalizer_configuration(&self, equalizer_configuration: EqualizerConfiguration) {
            self.equalizer
                .set_volumes(equalizer_configuration.volume_adjustments().into());
            let profile_index = self
                .profiles
                .get()
                .expect("profiles should have been intitialized already")
                .iter::<EqualizerProfileObject>()
                .position(|profile| {
                    profile.unwrap().profile_id() as u16 == equalizer_configuration.profile_id()
                })
                .unwrap_or(0)
                .try_into()
                .expect("could not convert usize to u32");
            self.profile_dropdown.set_selected(profile_index);
        }

        fn set_profiles(&self, profiles: Vec<EqualizerProfileObject>) {
            if let Some(model) = self.profiles.get() {
                model.remove_all();
                model.extend_from_slice(&profiles);

                self.custom_profile_index.set(
                    profiles
                        .iter()
                        .enumerate()
                        .find(|(_, profile)| {
                            profile.profile_id() as u16 == EqualizerConfiguration::CUSTOM_PROFILE_ID
                        })
                        .map(|(index, _)| index as u32),
                );
            }
        }

        fn set_up_custom_profile(&self) {
            self.set_up_custom_profile_selection_model();
            self.set_up_custom_profile_expression();
            self.set_up_custom_profile_item_factory();
            self.set_up_custom_profile_selection_changed_handler();
            self.set_up_custom_profile_create_delete_button();
        }

        fn set_up_custom_profile_selection_model(&self) {
            let model = gio::ListStore::new(CustomEqualizerProfileObject::static_type());
            self.custom_profile_dropdown.set_model(Some(&model));
            self.custom_profiles
                .set(model)
                .expect("set up should only run once");
        }

        fn set_up_custom_profile_expression(&self) {
            self.custom_profile_dropdown
                .set_expression(Some(PropertyExpression::new(
                    CustomEqualizerProfileObject::static_type(),
                    None::<Expression>,
                    "name",
                )));
        }

        fn set_up_custom_profile_item_factory(&self) {
            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let row = EqualizerProfileDropdownRow::new();
                list_item.set_child(Some(&row));
            });

            factory.connect_bind(move |_, list_item| {
                let equalizer_custom_profile_object = list_item
                    .item()
                    .expect("item must exist")
                    .downcast::<CustomEqualizerProfileObject>()
                    .expect("the item must be an EqualizerProfileObject");

                let row = list_item
                    .child()
                    .expect("must have a child")
                    .downcast::<EqualizerProfileDropdownRow>()
                    .expect("child must be a Box");

                row.set_name(equalizer_custom_profile_object.name());
                row.set_volume_adjustments(Some(
                    equalizer_custom_profile_object.volume_adjustments(),
                ));
            });
            self.custom_profile_dropdown.set_factory(Some(&factory));
        }

        fn set_up_custom_profile_selection_changed_handler(&self) {
            self.custom_profile_dropdown.connect_selected_item_notify(
            clone!(@weak self as this => move |_dropdown| {
                let maybe_selected_item = this.custom_profile_dropdown.selected_item()
                    .map(|item| item.downcast::<CustomEqualizerProfileObject>().unwrap());
                if let Some(selected_item) = maybe_selected_item {
                    this.sender.get().unwrap().send(Action::SelectCustomEqualizerProfile(selected_item.clone())).unwrap();
                    // Only apply settings if something changed from the perspective of the headphones
                    if !this.is_custom_profile() || this.equalizer.volume_adjustments() != selected_item.volume_adjustments() {
                        this.sender
                            .get()
                            .unwrap()
                            .send(Action::SetEqualizerConfiguration(this.equalizer_configuration()))
                            .unwrap();
                    }
                }
            }),
        );
        }

        fn set_up_custom_profile_create_delete_button(&self) {
            // Hide buttons if a preset profile is selected
            self.profile_dropdown
                .bind_property(
                    "selected-item",
                    &self.custom_profile_buttons.get(),
                    "visible",
                )
                .transform_to(|_, item: Option<EqualizerProfileObject>| {
                    item.map(|profile| {
                        profile.profile_id() as u16 == EqualizerConfiguration::CUSTOM_PROFILE_ID
                    })
                })
                .sync_create()
                .build();
            // Show create button if no custom profile is selected
            self.custom_profile_dropdown
                .bind_property(
                    "selected-item",
                    &self.create_custom_profile_button.get(),
                    "visible",
                )
                .transform_to(|_, item: Option<CustomEqualizerProfileObject>| Some(item.is_none()))
                .sync_create()
                .build();
            // Show delete button otherwise
            self.create_custom_profile_button
                .bind_property(
                    "visible",
                    &self.delete_custom_profile_button.get(),
                    "visible",
                )
                .invert_boolean()
                .sync_create()
                .build();
        }

        pub fn set_custom_profiles(&self, mut profiles: Vec<CustomEqualizerProfileObject>) {
            if let Some(model) = self.custom_profiles.get() {
                profiles.sort_unstable_by_key(|left| left.name());
                // Notifications need to be frozen to prevent the selection changes while removing and adding items from
                // causing the profile to change. We can't force having no selection when adding new items, so it
                // will change the selection to the newly added item. We can set it back to what it's supposed to be
                // afterwards.
                let _notify_freeze_guard = self.custom_profile_dropdown.freeze_notify();
                model.remove_all();
                model.extend_from_slice(&profiles);
                self.update_custom_profile_selection();
            }
        }

        fn update_custom_profile_selection(&self) {
            match self.custom_profiles.get() {
                Some(custom_profiles) if self.is_custom_profile() => {
                    let volumes = self.equalizer.volume_adjustments();
                    let custom_profile_index = custom_profiles
                        .iter::<CustomEqualizerProfileObject>()
                        .enumerate()
                        .find(|(_i, profile)| {
                            profile.as_ref().unwrap().volume_adjustments() == volumes
                        })
                        .map(|(i, _profile)| i as u32)
                        .unwrap_or(u32::MAX);

                    self.custom_profile_dropdown
                        .set_selected(custom_profile_index);
                }
                _ => {
                    self.custom_profile_dropdown.set_selected(u32::MAX);
                }
            }
        }

        fn set_up_preset_profile(&self) {
            self.set_up_preset_profile_selection_model();
            self.set_up_preset_profile_expression();
            self.set_up_preset_profile_item_factory();
            self.set_up_preset_profile_selection_changed_handler();
            self.set_up_preset_profile_items();
        }

        fn set_up_preset_profile_selection_model(&self) {
            let model = gio::ListStore::new(EqualizerProfileObject::static_type());
            self.profile_dropdown.set_model(Some(&model));
            self.profiles
                .set(model)
                .expect("set up should only run once");
        }

        fn set_up_preset_profile_expression(&self) {
            self.profile_dropdown
                .set_expression(Some(PropertyExpression::new(
                    EqualizerProfileObject::static_type(),
                    None::<Expression>,
                    "name",
                )));
        }

        fn set_up_preset_profile_item_factory(&self) {
            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let row = EqualizerProfileDropdownRow::new();
                list_item.set_child(Some(&row));
            });

            factory.connect_bind(move |_, list_item| {
                let equalizer_profile_object = list_item
                    .item()
                    .expect("item must exist")
                    .downcast::<EqualizerProfileObject>()
                    .expect("the item must be an EqualizerProfileObject");

                let row = list_item
                    .child()
                    .expect("must have a child")
                    .downcast::<EqualizerProfileDropdownRow>()
                    .expect("child must be a Box");

                row.set_name(equalizer_profile_object.name());

                let volume_adjustments =
                    PresetEqualizerProfile::from_id(equalizer_profile_object.profile_id() as u16)
                        .map(|profile| profile.volume_adjustments().adjustments());
                row.set_volume_adjustments(volume_adjustments);
            });
            self.profile_dropdown.set_factory(Some(&factory));
        }

        fn set_up_preset_profile_selection_changed_handler(&self) {
            self.profile_dropdown
            .connect_selected_item_notify(clone!(@weak self as this => move |_dropdown| {
                let selected_item: EqualizerProfileObject = this.profile_dropdown
                    .selected_item()
                    .expect("an item must be selected")
                    .downcast()
                    .expect("selected item must be an EqualizerProfileObject");
                let profile_id = selected_item.profile_id() as u16;
                let equalizer_configuration = if profile_id != EqualizerConfiguration::CUSTOM_PROFILE_ID {
                    let preset_profile = PresetEqualizerProfile::from_id(profile_id).unwrap_or_else(|| {
                        panic!("invalid preset profile id {profile_id}");
                    });
                    EqualizerConfiguration::new_from_preset_profile(preset_profile)
                } else {
                    EqualizerConfiguration::new_custom_profile(VolumeAdjustments::new(this.equalizer.volume_adjustments()))
                };
                this.set_equalizer_configuration(equalizer_configuration);
                this.update_custom_profile_selection();
                // TODO this is needed because this runs once during construction (before sender is set)
                // see if we can have sender get set before construction maybe?
                if let Some(sender) = this.sender.get() {
                    sender.send(Action::SetEqualizerConfiguration(equalizer_configuration)).unwrap();
                }
            }));
        }

        fn is_custom_profile(&self) -> bool {
            self.profile_dropdown
                .selected_item()
                .map(|item| {
                    item.downcast::<EqualizerProfileObject>()
                        .expect("must be EqualizerProfileObject")
                })
                .map(|profile| {
                    profile.profile_id() as u16 == EqualizerConfiguration::CUSTOM_PROFILE_ID
                })
                .unwrap_or(false)
        }

        fn set_up_preset_profile_items(&self) {
            let custom_profile_iter = [EqualizerProfileObject::new(
                "Custom",
                EqualizerConfiguration::CUSTOM_PROFILE_ID.into(),
            )]
            .into_iter();
            let preset_profile_iter = PresetEqualizerProfile::iter()
                .map(|preset| EqualizerProfileObject::new(&preset.to_string(), preset.id().into()));

            let profiles = custom_profile_iter
                .chain(preset_profile_iter)
                .collect::<Vec<_>>();
            self.set_profiles(profiles);
            self.profile_dropdown.set_selected(1); // Select Soundcore Signature by default
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EqualizerSettings {
        const NAME: &'static str = "OpenSCQ30EqualizerSettings";
        type Type = super::EqualizerSettings;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EqualizerSettings {
        fn constructed(&self) {
            self.parent_constructed();
            self.set_up_preset_profile();
            self.set_up_custom_profile();
        }
    }
    impl WidgetImpl for EqualizerSettings {}
    impl BoxImpl for EqualizerSettings {}
}

#[cfg(test)]
mod tests {
    use gtk::{
        glib::{MainContext, Priority},
        subclass::prelude::*,
        traits::WidgetExt,
    };
    use openscq30_lib::packets::structures::{
        EqualizerConfiguration, PresetEqualizerProfile, VolumeAdjustments,
    };

    use crate::objects::CustomEqualizerProfileObject;

    use super::EqualizerSettings;

    #[gtk::test]
    fn test_does_not_show_any_button_with_preset_profile_selected() {
        crate::load_resources();
        let settings = EqualizerSettings::new();
        let (sender, _receiver) = MainContext::channel(Priority::default());
        settings.set_sender(sender);
        settings.set_equalizer_configuration(EqualizerConfiguration::new_from_preset_profile(
            PresetEqualizerProfile::SoundcoreSignature,
        ));
        assert_eq!(
            false,
            settings.imp().create_custom_profile_button.is_visible(),
        );
        assert_eq!(
            false,
            settings.imp().delete_custom_profile_button.is_visible(),
        );
    }

    #[gtk::test]
    fn test_only_shows_create_button_with_no_custom_profile_selected() {
        crate::load_resources();
        let settings = EqualizerSettings::new();
        let (sender, _receiver) = MainContext::channel(Priority::default());
        settings.set_sender(sender);
        settings.set_equalizer_configuration(EqualizerConfiguration::new_custom_profile(
            VolumeAdjustments::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ));
        assert_eq!(
            true,
            settings.imp().create_custom_profile_button.is_visible(),
        );
        assert_eq!(
            false,
            settings.imp().delete_custom_profile_button.is_visible(),
        );
    }

    #[gtk::test]
    fn test_only_shows_delete_button_with_custom_profile_selected() {
        crate::load_resources();
        let settings = EqualizerSettings::new();
        let (sender, _receiver) = MainContext::channel(Priority::default());
        settings.set_sender(sender);
        settings.set_custom_profiles(vec![CustomEqualizerProfileObject::new(
            "test profile",
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        )]);
        settings.set_equalizer_configuration(EqualizerConfiguration::new_custom_profile(
            VolumeAdjustments::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ));
        assert_eq!(
            false,
            settings.imp().create_custom_profile_button.is_visible(),
        );
        assert_eq!(
            true,
            settings.imp().delete_custom_profile_button.is_visible(),
        );
    }
}
