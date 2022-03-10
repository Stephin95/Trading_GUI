mod fetcher;

mod style_custom;

use futures::TryFutureExt;
use iced::{container::Style, Application, Column, Command, Container, Element, Font, Row, Text};

use iced::button::{self, Button};
use iced::{executor, window, Color, Length};


use iced::{time, Settings, Subscription};
use std::sync::mpsc::sync_channel;

use tokio::task::JoinHandle;

use iced::{Align, Clipboard};
// use std::ops::Index;
use std::sync::mpsc::{self,Sender,SyncSender,Receiver};
// use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant};

use fetcher::Coin;
use fetcher::Message;
use fetcher::State;

// use std::sync::mpsc::Receiver;
use tokio::runtime::Runtime;
// #[derive(Default)]
//Creates mpsc sender and receiver transferring data between threads
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
        let (tx, rx) = sync_channel(1);
        CaptureMsg { tx: tx, rx: rx }
    }
}

// #[tokio::main]
// Entry to the app
fn main() -> iced::Result {
    let font = Font::External {
        name: "Icons",
        bytes: include_bytes!(r##"./Fonts/IBMPlexMono-Regular.ttf"##),
    };
    // getting mpsc sender and receiver
    let msg_capture: CaptureMsg = CaptureMsg::get_sync_capturers();
    // Running the app
    //with_flags are used to transfer data to the app loop
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
//initialising struct and values for running the app
struct App {
    table_result_row:usize,
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
    exit_sender:Sender<bool>,
}
// Application method unlike sandbox in iced rs allows to run async commands
impl Application for App {
    type Message = Message;
    type Flags = (Receiver<Vec<Coin>>, SyncSender<Vec<Coin>>);
    
    type Executor = executor::Default;
//Entry point function before the new gui is created
    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        println!("New Function called");

        println!("Finished Initialising the thread");
//Creating thread using tokio runtime and passing a clone of the sender to the thread

        let rt = Runtime::new().unwrap();
        let cloned_sender = _flags.1.clone();
        let (exit_mpsc_sender, exit_mpsc_receiver) = mpsc::channel();
        let handle = rt.spawn(fetcher::run(cloned_sender, String::from("bnbusdt"),exit_mpsc_receiver)); //BLZUSDT BNBUSDT
        //Getting initial values from the websocket thread as a vector of predetermined size and receiving it to the current thread and saving it to another vector
        let mut initial_vector = _flags.0.recv().expect("No Message from Thread");
        loop {
            println!("Adding values to display");
            let mut val = _flags.0.recv().expect("No Message from Thread");
            initial_vector.append(&mut val);
            if initial_vector.len() > 20 {
                break;
            }
        }
        
        //Adding default values to App struct here the initial values collected from the thread are added to the App struct along with other needed values for the GUI
        //For having unlimited lifetime ownership of the  sender and receiver of the mpsc is transferred to App Struct
        (
            Self {
                table_result_row:20,
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
                exit_sender:exit_mpsc_sender
            },

            // Command::perform(fetcher::get_result(self.reader),  |_| Message::Coin_details(())),
            Command::none(),
        )
    }
//Specifying title of the GUI
    fn title(&self) -> String {
        String::from("Trading GUI")
    }
//Update is called when a Message is returned from any of the following functions 
    fn update(&mut self, message: Message, clip: &mut Clipboard) -> Command<Message> {
//if statement to do any modification at the begining of the App

        if self.Start == true {
            self.Start = false;

        }
        //Handling Message based on the return value(message is an enum)

        match message {
            //no action could be used for future implementations
            Message::Coin_details(val) => {
            }
//This message is called by the subscription function which is currently set to call this as a loop
//Subscription function handles the clock functions in iced
//Here is the place where the value App.Received_Details is modified. This value is then used by the view function to add values to the GUI
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
                //Specifying maximum number of results to be displayed
                //  let result_display_length = 20;
                 //If the vector don't have enough elements to display then get values until the result_display_length+1 values is acheived
                if self.Received_Details.len() >= self.table_result_row {
                    for n in 0..received_value_length {
                        self.Received_Details.remove(0);
                    }
                }
                
                self.Received_Details.append(&mut res_msg);
                  }
                  //This message handles the key presses that changes the coin symbols. Each message will contain the name of the coin 
                  //from the button press
            Message::SplitButton(split_button_state, symbol) => {
                println!("split button pressed");
                //Auto update of the UI controlled by the subscription function turned off and 
                //kept idle until new values are added to the self.Received_Details vector
                self.state = State::Idle;
                self.exit_sender.send(true);

                println!("Killed the other threads");
                let (exit_mpsc_sender, exit_mpsc_receiver) = mpsc::channel();
                self.handles = self
                    .runtime
                    .spawn(fetcher::run(self.sender.clone(), symbol,exit_mpsc_receiver));
                self.exit_sender=exit_mpsc_sender;
                
                //Values are added to the vector until the length of the vector is enough to display
                let mut initial_vector = self.reader.recv().expect("No Message from Thread");
                loop {
                    println!("Adding values to display");
                    let mut val = self.reader.recv().expect("No Message from Thread");
                    initial_vector.append(&mut val);
                    if initial_vector.len() > self.table_result_row {
                        println!("BREAKING");
                        break;
                    }
                }
                //Previous values of the vector gets cleared 
                self.Received_Details.clear();
                println!("{:?}", &self.Received_Details);
                //New values of the different coin are added 
                self.Received_Details = initial_vector;
                println!("{:?}", &self.Received_Details);
                //self.state specifies if the clock function of the subscription should be used or not . 
                //Since the values are successfully fetched the app could now use the time function in the subscription function could start working
                self.state = State::Fetching {
                    last_tick: Instant::now(),
                };
                
            }
        }

        Command::none()
        
    }
    //subscription function is used to call something in a loop in this case it is used to fetch values from the other thread 
    //so that view function could update the interface
    fn subscription(&self) -> Subscription<Self::Message> {
        println!("Subscription called");
        match self.state {
            State::Idle => Subscription::none(),
            State::Fetching { .. } => {
                
                time::every(Duration::from_millis(120)).map(Message::TimeKeeper)
            }
        }
    }
    //view function controlls the gui of the element 

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        //creating the table with headers as headings which is specified here and then the items as rows 
        let table = Table::<Coin> {
            headers: vec![
                // Header {
                //     name: "Stream".to_string(),
                //     value: Box::new(|Coin| Coin.Stream.clone()),
                //     sort_value: Box::new(|Coin| Coin.Stream.clone()),
                // },
                // Header {
                //     name: "Symbol".to_string(),
                //     value: Box::new(|Coin| Coin.Symbol.clone()),
                //     sort_value: Box::new(|Coin| Coin.Symbol.clone()),
                // },
                // Header {
                //     name: "EventType".to_string(),
                //     value: Box::new(|Coin| Coin.EventType.clone()),
                //     sort_value: Box::new(|Coin| Coin.EventType.clone()),
                // },
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
                // Header {
                //     name: "Event Time".to_string(),
                //     value: Box::new(|Coin| Coin.EventTime.to_string()),
                //     sort_value: Box::new(|Coin| Coin.EventTime.to_string()),
                // },
            ],
            
            items: &self.Received_Details,
        };
        //Temporary hard coded buttton to access values of a particular crypto currency
        let split_button = Button::new(&mut self.split_button.0, Text::new("BTCUSDT")).style(style_custom::dark::Button).on_press(
            Message::SplitButton(button::State::new(), String::from("btcusdt")),
        );
        //Temporary hard coded buttton to access values of a particular crypto currency
        let split_button2 = Button::new(&mut self.split_button.1, Text::new("BNBUSDT")).style(style_custom::dark::Button).on_press(
            Message::SplitButton(button::State::new(), String::from("bnbusdt")),
        );
        let active_coin_label=Text::new(self.Received_Details[0].Symbol.to_string()).size(50).color(Color::WHITE);
        
        //column member which distributes it sub member in to column
        let side_pane_col=Column::new().push(split_button).push(split_button2).align_items(Align::Center);
        let side_pane_col_container=Container::new(side_pane_col).height(Length::Units(100)).style(style_custom::SidePane).padding(5).center_x()
        .center_y();
        let main_side_pane=Column::new().push(active_coin_label).push(side_pane_col_container);
        let main_side_pane_container=Container::new(main_side_pane).style(style_custom::SidePane).height(Length::Fill).padding(5);
        //adding the column member and the table.view in Row which distributes it's children horizontally
        let app_row = Row::with_children(vec![ main_side_pane_container.into(),table.view(),])
            .spacing(10)
            .align_items(Align::Center).width(Length::Fill)
            .height(Length::Fill);
        //Row is contained inside a Container and is returned
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

//table view implementation
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
            .align_x(Align::Center)
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

        let table = Column::new().push(headers).push(rows).align_items(Align::Center);
        Container::new(table)
            .style(style_custom::Theme::Dark)
            .into()
    }
}

