// SPDX-License-Identifier: MIT

use crate::book::{Book, load_data};
use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{self, Horizontal, Vertical};
use cosmic::iced::{Color, Length, Subscription};
use cosmic::iced_core::Text;
use cosmic::iced_wgpu::graphics::text::cosmic_text;
use cosmic::iced_widget::{Stack, stack};
use cosmic::widget::button::text;
use cosmic::widget::icon::Handle;
use cosmic::widget::{self, about::About, icon, menu, nav_bar};
use cosmic::widget::{container, scrollable, svg};
use cosmic::{iced_core, iced_futures, prelude::*};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::time::Duration;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Time active
    time: u32,
    /// Toggle the watch subscription
    watch_is_active: bool,

    books: Vec<Book>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    ToggleWatch,
    UpdateConfig(Config),
    WatchTick(u32),
    MouseEnterShortDescription(usize),
    MouseExitShortDescription(usize),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.dechro.antiquar";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("books"))
            .data::<Page>(Page::Books)
            .icon(
                icon::from_svg_bytes(include_bytes!("../assets/icons/library-big.svg"))
                    .symbolic(true)
                    .icon(),
            )
            .activate();

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => {
                    // for why in errors {
                    //     tracing::error!(%why, "error loading app config");
                    // }

                    config
                }
            })
            .unwrap_or_default();

        let books = load_data(std::path::Path::new(&String::from(
            config.clone().data_path,
        )));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config,
            time: 0,
            watch_is_active: false,
            books,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::Books => {
                let mut books: Vec<_> = self
                    .books
                    .iter()
                    .enumerate()
                    .map(|item| {
                        let title = title(item.1);
                        let author = author(item.1);
                        let date = date(item.1);

                        let description = widget::row().push(
                            widget::mouse_area(
                                widget::text(item.1.data.clone().unwrap().description)
                                    .wrapping(iced_core::text::Wrapping::Glyph),
                            )
                            .on_enter(Message::MouseEnterShortDescription(item.0))
                            .on_exit(Message::MouseExitShortDescription(item.0)),
                        );
                        let mut button = None;

                        if item.1.description_hovered {
                            button = Some(container(
                                widget::button::text(fl!("expand-description"))
                                    .trailing_icon(
                                        icon::from_svg_bytes(include_bytes!(
                                            "../assets/icons/chevrons-up-down.svg"
                                        ))
                                        .symbolic(true),
                                    )
                                    .class(widget::button::ButtonClass::Suggested),
                            ))
                        };

                        let description = Stack::new().push(description).push_maybe(button);

                        container(
                            widget::column()
                                .push(title)
                                .push(
                                    widget::row()
                                        .push(author)
                                        .push(date)
                                        .push(widget::horizontal_space())
                                        .push(widget::Space::with_width(
                                            Theme::default().cosmic().space_xxs(),
                                        ))
                                        .push(description)
                                        .spacing(Theme::default().cosmic().space_xxs()),
                                )
                                .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .height(Theme::default().cosmic().space_xl())
                    })
                    .flat_map(|item| [container(widget::divider::horizontal::default()), item])
                    .collect();

                if books.len() > 0 {
                    books.remove(0);
                }

                let book_list = widget::column().append(&mut books);
                let table = scrollable(book_list);

                widget::column::with_capacity(1)
                    .push(table)
                    .height(Length::Fill)
                    .into()
            }
        };

        widget::container(content)
            .width(Length::Fill)
            .padding(Theme::default().cosmic().space_m())
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Conditionally enables a timer that emits a message every second.
        if self.watch_is_active {
            subscriptions.push(Subscription::run(|| {
                iced_futures::stream::channel(1, |mut emitter| async move {
                    let mut time = 1;
                    let mut interval = tokio::time::interval(Duration::from_secs(1));

                    loop {
                        interval.tick().await;
                        _ = emitter.send(Message::WatchTick(time)).await;
                        time += 1;
                    }
                })
            }));
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::WatchTick(time) => {
                self.time = time;
            }

            Message::ToggleWatch => {
                self.watch_is_active = !self.watch_is_active;
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::MouseEnterShortDescription(index) => {
                set_description_hovered(true, &mut self.books[index]);
            }

            Message::MouseExitShortDescription(index) => {
                set_description_hovered(false, &mut self.books[index]);
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_title()
    }
}

fn set_description_hovered(hovered: bool, book: &mut Book) {
    book.description_hovered = hovered;
}

fn date(item: &Book) -> cosmic::iced_widget::Row<'_, Message, Theme> {
    widget::row()
        .push(
            svg::Svg::new(iced_core::svg::Handle::from_memory(include_bytes!(
                "../assets/icons/calendar.svg"
            )))
            .class(cosmic::theme::Svg::custom(|theme| svg::Style {
                color: Some(theme.cosmic().on_bg_color().into()),
            }))
            .width(20)
            .height(20),
        )
        .push(widget::text(item.data.clone().unwrap().year.to_string()))
        .spacing(Theme::default().cosmic().space_xxxs())
}

fn author(item: &Book) -> cosmic::iced_widget::Row<'_, Message, Theme> {
    widget::row()
        .push(
            svg::Svg::new(iced_core::svg::Handle::from_memory(include_bytes!(
                "../assets/icons/circle-user-round.svg"
            )))
            .class(cosmic::theme::Svg::custom(|theme| svg::Style {
                color: Some(theme.cosmic().on_bg_color().into()),
            }))
            .width(20)
            .height(20),
        )
        .push(widget::text(item.data.clone().unwrap().author))
        .width(Length::Shrink)
        .spacing(Theme::default().cosmic().space_xxxs())
}

fn title(item: &Book) -> widget::Text<'_, Theme, Renderer> {
    widget::text::heading(item.data.clone().unwrap().title)
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

/// The page to display in the application.
pub enum Page {
    Books,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
