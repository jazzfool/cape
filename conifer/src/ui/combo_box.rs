use crate::*;
use cape::{
    node::{Interaction, MouseButton},
    state::{use_state, Accessor, StateAccessor},
    ui::{self, NodeLayout},
    Sides2,
};
use std::rc::Rc;

pub type ComboBox<Centre, Popup, Item> = StackLayout<(
    ui::Interact<StackLayout<(Centre, Item)>>,
    Option<StackLayout<(Popup, ColumnLayout<(Vec<ui::Interact<Item>>,)>)>>,
)>;

pub struct ComboBoxProps<Centre, Popup, Item>
where
    Centre: 'static + ui::Merge + ui::Expand,
    Popup: 'static + ui::Merge + ui::Expand,
    Item: 'static + Clone + ui::Merge + ui::Expand,
{
    pub centre: Centre,
    pub popup: Popup,
    pub items: Vec<Item>,
    pub index: usize,
    pub on_change: Rc<dyn Fn(usize)>,
    pub centre_padding: Sides2,
    pub centre_item: StackItem,
}

impl<Centre, Popup, Item> Default for ComboBoxProps<Centre, Popup, Item>
where
    Centre: 'static + Default + ui::Merge + ui::Expand,
    Popup: 'static + Default + ui::Merge + ui::Expand,
    Item: 'static + Default + Clone + ui::Merge + ui::Expand,
{
    fn default() -> Self {
        ComboBoxProps {
            centre: Default::default(),
            popup: Default::default(),
            items: Default::default(),
            index: 0,
            on_change: Rc::new(|_| {}),
            centre_padding: Default::default(),
            centre_item: Default::default(),
        }
    }
}

impl<Centre, Popup, Item> Props<ComboBox<Centre, Popup, Item>>
    for ComboBoxProps<Centre, Popup, Item>
where
    Centre: 'static + Default + ui::Merge + ui::Expand,
    Popup: 'static + Default + ui::Merge + ui::Expand,
    Item: 'static + Default + Clone + ui::Merge + ui::Expand,
{
    #[cape::ui]
    fn build(self) -> ComboBox<Centre, Popup, Item> {
        let opened = use_state(|| false);

        let on_change = self.on_change;

        Stack::default().children((
            ui::Interact::new(
                Stack {
                    margin: self.centre_padding,
                    ..Default::default()
                }
                .children((
                    self.centre,
                    (
                        self.items
                            .get(self.index)
                            .cloned()
                            .expect("invalid item index"),
                        self.centre_item,
                    ),
                )),
                move |e| {
                    if let Interaction::MouseDown {
                        button: MouseButton::Left,
                        ..
                    } = e
                    {
                        opened.set(!opened.get());
                    }
                },
                Default::default(),
                false,
            ),
            if opened.get() {
                Some(
                    Stack::default().children((
                        self.popup,
                        Column::default().children((self
                            .items
                            .into_iter()
                            .enumerate()
                            .map(move |(i, item)| {
                                let on_change = Rc::clone(&on_change);
                                ui::Interact::new(
                                    item,
                                    move |e| {
                                        if let Interaction::MouseDown {
                                            button: MouseButton::Left,
                                            ..
                                        } = e
                                        {
                                            on_change(i);
                                            opened.set(false);
                                        }
                                    },
                                    Default::default(),
                                    false,
                                )
                            })
                            .collect::<Vec<_>>(),)),
                    )),
                )
            } else {
                None
            },
        ))
    }
}

impl<Centre, Popup, Item> ComboBoxProps<Centre, Popup, Item>
where
    Centre: 'static + ui::Merge + ui::Expand,
    Popup: 'static + ui::Merge + ui::Expand,
    Item: 'static + Clone + ui::Merge + ui::Expand,
{
    pub fn centre(mut self, centre: Centre) -> Self {
        self.centre = centre;
        self
    }

    pub fn popup(mut self, popup: Popup) -> Self {
        self.popup = popup;
        self
    }

    pub fn items(mut self, items: Vec<Item>) -> Self {
        self.items = items;
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn on_change(mut self, on_change: impl Fn(usize) + 'static) -> Self {
        self.on_change = Rc::new(on_change);
        self
    }

    pub fn state(self, state: StateAccessor<usize>) -> Self {
        self.index(state.get()).on_change(move |i| state.set(i))
    }

    pub fn centre_padding(mut self, centre_padding: Sides2) -> Self {
        self.centre_padding = centre_padding;
        self
    }

    pub fn centre_item(mut self, centre_item: StackItem) -> Self {
        self.centre_item = centre_item;
        self
    }
}

pub fn combo_box<Centre, Popup, Item>(
    props: ComboBoxProps<Centre, Popup, Item>,
) -> ComboBox<Centre, Popup, Item>
where
    Centre: 'static + Default + ui::Merge + ui::Expand,
    Popup: 'static + Default + ui::Merge + ui::Expand,
    Item: 'static + Default + Clone + ui::Merge + ui::Expand,
{
    props.build()
}
