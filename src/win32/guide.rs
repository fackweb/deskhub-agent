use crate::types;
use crate::utils;

use super::service;
use iced::widget::{button, image, row, text, Column, Space};
use iced::{color, executor, Theme};
use iced::{theme, Application, Command};
use iced::{Alignment, Element, Font, Length};
use once_cell::sync::Lazy;
use std::env;
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SUCCESS};

static DESK_SERVICE: Lazy<Arc<Mutex<service::Service>>> =
    Lazy::new(|| Arc::new(Mutex::new(service::Service::new(types::DESK_SEVICE_NAME))));

#[derive(Debug, Clone)]
pub enum Message {
    RegisterButtonPressed,
    StopServiceButtonPressed,
    StartServiceButtonPressed,
    RemoveServiceButtonPressed,
    Spining(bool),
    AlertUpdated(Option<types::Alert>, bool),
    ServiceStatusUpdated(service::ServiceStatus),
}

pub struct GuideWindow {
    service_status: service::ServiceStatus,
    alert: Option<types::Alert>,
    spining: bool,
}

fn commands_with_spining(
    spining: bool,
    message: String,
    cmd: Command<Message>,
) -> Command<Message> {
    let spining_cmd = Command::perform(async move { Message::Spining(spining) }, |message| message);
    let alert_cmd = Command::perform(
        async {
            Message::AlertUpdated(
                Some(types::Alert {
                    message,
                    alert_type: types::AlertType::Info,
                }),
                false,
            )
        },
        |message| message,
    );
    Command::batch([spining_cmd, alert_cmd, cmd])
}

impl Application for GuideWindow {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        //The program must have the permission to open and operate the Service.
        let open_result = DESK_SERVICE.lock().unwrap().open();
        let service_status = if open_result != ERROR_SUCCESS {
            if open_result == ERROR_SERVICE_DOES_NOT_EXIST {
                service::ServiceStatus::DoesNotExist
            } else {
                //Apart from the known error DoesNotExist, explain that it is not possible to operate on the Service.
                service::ServiceStatus::Unknown
            }
        } else {
            service::ServiceStatus::Querying
        };
        let command: Command<Message> = if service_status == service::ServiceStatus::Querying {
            commands_with_spining(
                true,
                "Checking service status...".to_string(),
                Command::perform(
                    async {
                        let status: service::ServiceStatus =
                            DESK_SERVICE.lock().unwrap().query_status();
                        Ok::<service::ServiceStatus, std::convert::Infallible>(status)
                    },
                    |result| match result {
                        Ok(status) => Message::ServiceStatusUpdated(status),
                        Err(e) => Message::AlertUpdated(
                            Some(types::Alert {
                                message: e.to_string(),
                                alert_type: types::AlertType::Error,
                            }),
                            true,
                        ),
                    },
                ),
            )
        } else {
            Command::none()
        };

        (
            GuideWindow {
                service_status,
                alert: None,
                spining: false,
            },
            command,
        )
    }

    fn title(&self) -> String {
        String::from("DeskHub")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::Spining(spining) => {
                self.spining = spining;
                Command::none()
            }
            Message::ServiceStatusUpdated(status) => {
                self.service_status = status;
                self.spining = false;
                self.alert = None;
                Command::none()
            }
            Message::RegisterButtonPressed => commands_with_spining(
                true,
                "Registering service...".to_string(),
                Command::perform(
                    async {
                        let mut service = DESK_SERVICE.lock().unwrap();
                        if let Some(mut execute_path) = utils::get_executable_path() {
                            execute_path = format!("\"{}\"", execute_path);
                            execute_path.push_str(" -service");
                            let successed = service.register("DeskHubService", &execute_path);
                            if !successed {
                                return Err("Service registration failed.".to_string());
                            }
                            let status = service.query_status();
                            return Ok::<service::ServiceStatus, String>(status);
                        }
                        Err("Failed to get the execution path.".to_string())
                    },
                    |result| match result {
                        Ok(status) => Message::ServiceStatusUpdated(status),
                        Err(e) => Message::AlertUpdated(
                            Some(types::Alert {
                                message: e,
                                alert_type: types::AlertType::Error,
                            }),
                            true,
                        ),
                    },
                ),
            ),
            Message::StopServiceButtonPressed => commands_with_spining(
                true,
                "Service stopping".to_string(),
                Command::perform(
                    async {
                        let service = DESK_SERVICE.lock().unwrap();
                        service.stop();
                        let status = service.query_status();
                        if status != service::ServiceStatus::Stopped {
                            return Err("Failed to stop the service".to_string());
                        }
                        Ok::<service::ServiceStatus, String>(status)
                    },
                    |result| match result {
                        Ok(status) => Message::ServiceStatusUpdated(status),
                        Err(e) => Message::AlertUpdated(
                            Some(types::Alert {
                                message: e,
                                alert_type: types::AlertType::Error,
                            }),
                            true,
                        ),
                    },
                ),
            ),
            Message::StartServiceButtonPressed => commands_with_spining(
                true,
                "The service is starting up...".to_string(),
                Command::perform(
                    async {
                        let service = DESK_SERVICE.lock().unwrap();
                        service.start();
                        let status = service.query_status();
                        if status != service::ServiceStatus::Running {
                            return Err("Failed to start the service".to_string());
                        }
                        Ok::<service::ServiceStatus, String>(status)
                    },
                    |result| match result {
                        Ok(status) => Message::ServiceStatusUpdated(status),
                        Err(e) => Message::AlertUpdated(
                            Some(types::Alert {
                                message: e,
                                alert_type: types::AlertType::Error,
                            }),
                            true,
                        ),
                    },
                ),
            ),
            Message::RemoveServiceButtonPressed => commands_with_spining(
                true,
                "Removing service...".to_string(),
                Command::perform(
                    async {
                        let mut service = DESK_SERVICE.lock().unwrap();
                        if service.unregister() {
                            return Ok(service::ServiceStatus::DoesNotExist);
                        } else {
                            return Err("Service removal fail".to_string());
                        }
                    },
                    |result| match result {
                        Ok(status) => Message::ServiceStatusUpdated(status),
                        Err(e) => Message::AlertUpdated(
                            Some(types::Alert {
                                message: e,
                                alert_type: types::AlertType::Error,
                            }),
                            true,
                        ),
                    },
                ),
            ),
            Message::AlertUpdated(alert, hide_spining) => {
                self.alert = alert;
                if hide_spining {
                    self.spining = false;
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut columns = Column::new();
        columns = columns
            .push(image(format!("{}/assets/wave.png", env!("CARGO_MANIFEST_DIR"))).width(30))
            .push(text("Hey!").size(24).font(Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }))
            .push(Space::with_height(15));

        match self.service_status {
            service::ServiceStatus::Querying => {
                let txt = "Querying service status....";
                columns = columns.push(text(txt).size(16));
            }
            service::ServiceStatus::DoesNotExist => {
                let txt = "DeskHub needs to be registered as a service before running.\nPlease make sure not to move the program to a different location after clicking on registration, otherwise the service will not run properly and will require re-registration.";
                columns = columns
                    .push(text(txt).size(16))
                    .push(Space::with_height(15))
                    .push((|| {
                        let btn = button("Register as service")
                            .padding([12, 24])
                            .style(theme::Button::Primary);
                        if self.spining {
                            btn
                        } else {
                            btn.on_press(Message::RegisterButtonPressed)
                        }
                    })());
            }
            service::ServiceStatus::Running => {
                let txt = "DeskHub Service has running. \nA window program displaying test information will appear normally. \nIf there are any issues, please restart the service or reinstall it after deleting the service.";
                columns = columns
                    .push(text(txt).size(16))
                    .push(Space::with_height(15))
                    .push((|| {
                        let stop_btn = button("Stop service")
                            .padding([8, 14])
                            .style(theme::Button::Primary);
                        let remove_btn = button("Remove service")
                            .padding([8, 14])
                            .style(theme::Button::Destructive);
                        if self.spining {
                            row![stop_btn, Space::with_width(15), remove_btn]
                        } else {
                            row![
                                stop_btn.on_press(Message::StopServiceButtonPressed),
                                Space::with_width(15),
                                remove_btn.on_press(Message::RemoveServiceButtonPressed)
                            ]
                        }
                    })());
            }
            service::ServiceStatus::Stopped => {
                let txt = "Service has stopped. \nOnce the service is running normally, the main interface will automatically open.";
                columns = columns
                    .push(text(txt).size(16))
                    .push(Space::with_height(15))
                    .push((|| {
                        let start_btn = button("Start service")
                            .padding([8, 14])
                            .style(theme::Button::Primary);
                        let remove_btn = button("Remove service")
                            .padding([8, 14])
                            .style(theme::Button::Destructive);
                        if self.spining {
                            row![start_btn, Space::with_width(15), remove_btn]
                        } else {
                            row![
                                start_btn.on_press(Message::StartServiceButtonPressed),
                                Space::with_width(15),
                                remove_btn.on_press(Message::RemoveServiceButtonPressed)
                            ]
                        }
                    })());
            }
            service::ServiceStatus::Unknown => {
                println!("hello2");
            }
        }

        if let Some(alert) = self.alert.as_ref() {
            columns = columns
                .push(Space::with_height(15))
                .push(text(&alert.message).size(16).style(color!(0xFF0000)));
        }

        columns
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_items(Alignment::Start)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::default()
    }

    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        <Self::Theme as iced::application::StyleSheet>::Style::default()
    }
}
