mod fetcher;

mod style_custom;

use iced::{container::Style, Application, Column, Command, Container, Element, Font, Row, Text};

use iced::button::{self, Button};
use iced::{executor, window, Color, Length};


use iced::{time, Settings, Subscription};
use std::sync::mpsc::sync_channel;

use tokio::task::JoinHandle;

use iced::{Align, Clipboard};
// use std::ops::Index;
use std::sync::mpsc::SyncSender;
// use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant};

use fetcher::Coin;
use fetcher::Message;
use fetcher::State;

use std::sync::mpsc::Receiver;
use tokio::runtime::Runtime;
// #[derive(Default)]
struct CaptureMsg {
    tx: SyncSender<Vec<Coin>>,
    rx: Receiver<Vec<Coin>>,
}
impl CaptureMsg {
    // pub fn get_capturers() -> Self {
    //     let (mut tx, mut rx) = mpsc::channel();
    //     CaptureMsg { tx: tx, rx: rx }
    // }
    pub fn get_sync_capturers() -> Self {
        let (tx, rx) = sync_channel(0);
        CaptureMsg { tx: tx, rx: rx }
    }
}

// #[tokio::main]
fn main() -> iced::Result {
    let font = Font::External {
        name: "Icons",
        bytes: include_bytes!(r##"./Fonts/IBMPlexMono-Regular.ttf"##),
    };
    let msg_capture: CaptureMsg = CaptureMsg::get_sync_capturers();

    App::run(Settings {
        antialiasing: false,
        window: window::Settings {
            // position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::with_flags((msg_capture.rx, msg_capture.tx))
    })

}

// #[derive(Default)]
struct App {
    history: Vec<Coin>,
    theme: style_custom::Theme,
    Start: bool,
    headers: Vec<String>,
    Received_Details: Vec<Coin>,
    update_status_button: button::State,
    reader: Receiver<Vec<Coin>>,
    sender: SyncSender<Vec<Coin>>,
    state: State,
    handles: JoinHandle<()>,
    runtime: Runtime,
    split_button: (button::State,button::State),
}

impl Application for App {
    type Message = Message;
    type Flags = (Receiver<Vec<Coin>>, SyncSender<Vec<Coin>>);
    
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        println!("New Function called");

        println!("Finished Initialising the thread");


        let rt = Runtime::new().unwrap();
        let cloned_sender = _flags.1.clone();
        let handle = rt.spawn(fetcher::run(cloned_sender, String::from("bnbusdt"))); //BLZUSDT BNBUSDT
        let mut initial_vector = _flags.0.recv().expect("No Message from Thread");
        loop {
            println!("Adding values to display");
            let mut val = _flags.0.recv().expect("No Message from Thread");
            initial_vector.append(&mut val);
            if initial_vector.len() > 20 {
                break;
            }
        }
        
        (
            Self {
                history: Default::default(),
                theme: Default::default(),
                Start: true,
                headers: vec![
                    String::from("Type"),
                    String::from("Value1"),
                    String::from("TypeValue2"),
                ],
                Received_Details: initial_vector,
                update_status_button: button::State::new(),
                reader: _flags.0,
                sender: _flags.1,
                state: State::Fetching {
                    last_tick: Instant::now()
                },
                handles: handle,
                runtime: rt,
                split_button: (button::State::new(),button::State::new()),
            },

            // Command::perform(fetcher::get_result(self.reader),  |_| Message::Coin_details(())),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Trading GUI")
    }

    fn update(&mut self, message: Message, clip: &mut Clipboard) -> Command<Message> {


        if self.Start == true {
            self.Start = false;

        }

        match message {
            Message::Coin_details(val) => {
            }

            Message::TimeKeeper(val) => {
                
                print!("Subscription--> Time Keeper\n");
                let msg = self.reader.recv();
                let mut res_msg = match msg {
                    Ok(val) => val,
                    Err(val) => {
                        println!("Error occured: Null value received from the other thread");
                        vec![Coin::get_nul_values()]
                    }
                };
                print!("Data Received\n");
                
                let received_value_length = *&res_msg.len() as i32;
                 let result_display_length = 20;
                if self.Received_Details.len() >= result_display_length {
                    for n in 0..received_value_length {
                        self.Received_Details.remove(0);
                    }
                }
                
                self.Received_Details.append(&mut res_msg);
                  }
            Message::SplitButton(split_button_state, symbol) => {
                println!("split button pressed");
                self.state = State::Idle;
                
                
                {
                self.handles.abort();
                self.handles.abort();
                }

                println!("Killed the other threads");

                self.handles = self
                    .runtime
                    .spawn(fetcher::run(self.sender.clone(), symbol));
                
                let mut initial_vector = self.reader.recv().expect("No Message from Thread");
                loop {
                    println!("Adding values to display");
                    let mut val = self.reader.recv().expect("No Message from Thread");
                    initial_vector.append(&mut val);
                    if initial_vector.len() > 20 {
                        println!("BREAKING");
                        break;
                    }
                }
                self.Received_Details.clear();
                println!("{:?}", &self.Received_Details);
                self.Received_Details = initial_vector;
                println!("{:?}", &self.Received_Details);
                self.state = State::Fetching {
                    last_tick: Instant::now(),
                };
                
            }
        }

        Command::none()
        
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        println!("Subscription called");
        match self.state {
            State::Idle => Subscription::none(),
            State::Fetching { .. } => {
                
                time::every(Duration::from_millis(120)).map(Message::TimeKeeper)
            }
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let table = Table::<Coin> {
            headers: vec![
                Header {
                    name: "Stream".to_string(),
                    value: Box::new(|Coin| Coin.Stream.clone()),
                    sort_value: Box::new(|Coin| Coin.Stream.clone()),
                },
                Header {
                    name: "Symbol".to_string(),
                    value: Box::new(|Coin| Coin.Symbol.clone()),
                    sort_value: Box::new(|Coin| Coin.Symbol.clone()),
                },
                Header {
                    name: "EventType".to_string(),
                    value: Box::new(|Coin| Coin.EventType.clone()),
                    sort_value: Box::new(|Coin| Coin.EventType.clone()),
                },
                Header {
                    name: "BidPriceLevel".to_string(),
                    value: Box::new(|Coin| Coin.BidPriceLevel.clone()),
                    sort_value: Box::new(|Coin| Coin.BidPriceLevel.clone()),
                },
                Header {
                    name: "BidQuantity".to_string(),
                    value: Box::new(|Coin| Coin.BidQuantity.clone()),
                    sort_value: Box::new(|Coin| Coin.BidQuantity.clone()),
                },
                Header {
                    name: "AsksPriceLevel".to_string(),
                    value: Box::new(|Coin| Coin.AsksPriceLevel.clone()),
                    sort_value: Box::new(|Coin| Coin.AsksPriceLevel.clone()),
                },
                Header {
                    name: "AsksQuantity".to_string(),
                    value: Box::new(|Coin| Coin.AsksQuantity.clone()),
                    sort_value: Box::new(|Coin| Coin.AsksQuantity.clone()),
                },
                Header {
                    name: "Event Time".to_string(),
                    value: Box::new(|Coin| Coin.EventTime.to_string()),
                    sort_value: Box::new(|Coin| Coin.EventTime.to_string()),
                },
            ],
            
            items: &self.Received_Details,
        };

        let split_button = Button::new(&mut self.split_button.0, Text::new("BTCUSDT")).on_press(
            Message::SplitButton(button::State::new(), String::from("btcusdt")),
        );
        let split_button2 = Button::new(&mut self.split_button.1, Text::new("BNBUSDT")).on_press(
            Message::SplitButton(button::State::new(), String::from("bnbusdt")),
        );
        let side_pane_col=Column::new().push(split_button).push(split_button2);

        let app_row = Row::with_children(vec![ side_pane_col.into(),table.view(),])
            .spacing(10)
            .align_items(Align::Center).width(Length::Fill)
            .height(Length::Fill);

        Container::new(app_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style_custom::Cell)
            .center_x()
            .center_y()
            .padding(0)
            .into()
    }
}

pub struct Header<T> {
    pub name: String,
    pub value: Box<dyn Fn(&T) -> String>,
    pub sort_value: Box<dyn Fn(&T) -> String>,
}

impl<T> std::fmt::Debug for Header<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HeaderConfig<{}> {{ name: {} }}",
            std::any::type_name::<T>(),
            self.name
        )
    }
}
#[derive(Debug)]
pub struct Table<'a, T> {
    pub headers: Vec<Header<T>>,
    pub items: &'a Vec<T>,
}

impl<'a, T> Table<'a, T> {
    fn cell<'element>(text: &str) -> Element<'element, Message> {
        Container::new(Text::new(text.to_string()))
            .style(style_custom::Cell)
            .width(iced::Length::Fill)
            .align_y(Align::Center)
            .into()
    }
    pub fn view<'element>(&self) -> Element<'element, Message> {
        let headers = self
            .headers
            .iter()
            .fold(Row::new(), |row, header| row.push(Self::cell(&header.name)))
            .spacing(10);
        let rows = self.items.iter().fold(Column::new(), |column, item| {
            column.push(
                self.headers
                    .iter()
                    .fold(Row::new(), |row, Header { value, .. }| {
                        row.push(Self::cell(&value(item)))
                            .align_items(Align::Center)
                    })
                    .spacing(10),
            )
        });
        // let mut button_state = button::State::new();

        let table = Column::new().push(headers).push(rows);
        Container::new(table)
            .style(style_custom::Theme::Dark)
            .into()
    }
}

