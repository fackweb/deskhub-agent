use iced::widget::{column, text, Space};
use iced::{executor, Theme};
use iced::{Alignment, Element, Length};
use iced::{Application, Command};

#[derive(Debug, Clone)]
pub enum Message {}
pub struct DeskWindow {}

impl Application for DeskWindow {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (DeskWindow {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("DeskHub")
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        column![
            Space::with_height(30),
            text("This is just a test program.").size(16)
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .align_items(Alignment::Start)
        .into()
    }

    fn update(&mut self, _: Self::Message) -> Command<Self::Message> {
        Command::none()
    }
}
