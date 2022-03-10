use async_tungstenite::stream::Stream;
use async_tungstenite::{
    tokio::{connect_async, TokioAdapter},
    WebSocketStream,
};
use futures::{prelude::*, stream::SplitStream};
use json::{Error, JsonValue};

use iced::button;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{sync::mpsc::SyncSender, time::Instant};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
// use std::{time, sync::mpsc::{SyncSender}};

pub async fn run(tx: SyncSender<Vec<Coin>>, symbol: String, exit_receiver_ref: Receiver<bool>) {
    println!("Fetching values from the server");

    let web_socket_url: String =
        String::from("wss://fstream.binance.com/stream?streams=SYMBOL@depth@100ms")
            .replace("SYMBOL", &symbol);

    println!("Fetching from {:?}", &web_socket_url);
    #[cfg(not(any(
        feature = "async-tls",
        feature = "tokio-native-tls",
        feature = "tokio-openssl"
    )))]
    let (ws_stream, _) = connect_async(web_socket_url)
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    let (_write, mut reader) = ws_stream.split();

    loop {
        let max_row = 1;
        let mut coin_sheet = Vec::with_capacity(max_row);

        let mut row = 0;
        while row < max_row {
            let json_data = get_result(&mut reader).await;
            row = row + 1;
            coin_sheet.push(json_data);
        }
        // print!("Sending data{:?}", &coin_sheet);
        // thread::sleep(sleep_time);

        match exit_receiver_ref.try_recv() {
            Ok(status) => {
                print!("Breaking Loop");
                break;
            }
            Err(_) => {
                println!("Continuing.");
                // break;
            }
        };
        println!("Sending data");

        tx.send(coin_sheet).unwrap();
    }
}

pub async fn get_result(
    reader: &mut SplitStream<
        WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
    >,
) -> Coin {
    let res = reader
        .next()
        .await
        .expect("Error getting value from the websocket on first unwrap")
        .expect("Error getting value from the websocket on second unwrap");
    // println!("{:?}",&res);
    let parsed_json_result = clean_json(res);
    let json_to_coin = match parsed_json_result {
        Ok(json_val) => get_values(json_val),
        Err(_err_val) => Coin::get_nul_values(),
    };

    json_to_coin
}

fn clean_json(json_message: async_tungstenite::tungstenite::Message) -> Result<JsonValue, Error> {
    let cleaned_string = json_message
        .into_text()
        .expect("Error converting tungstenite message to string");
    // println!("{:?}", cleaned_string);
    let parsed_json = json::parse(&cleaned_string);
    // println!("{:?}",&cleaned_string);
    parsed_json
    // json::parse(&cleaned_string).expect("Error converting the websocket message to json")
}

fn get_values(mut parsed_json_object: JsonValue) -> Coin {
    //Stream
    // println!("{:?}",parsed_json_object.pretty(4));
    // println!("{:?}",parsed_json_object.dump());
    // println!("{}", &parsed_json_object["stream"]);
    let stream = &parsed_json_object["stream"]
        .take_string()
        .expect("Error unwraping the parsed json object from the given key");
    // println!("{}", stream);
    let event_type = &parsed_json_object["data"]["e"]
        .take_string()
        .expect("Error unwraping the parsed json object from the given key");
    // println!("{}", event_type);
    // Symbol
    let symbol = &parsed_json_object["data"]["s"]
        .take_string()
        .expect("Error unwraping the parsed json object from the given key");

    let b_price_level = match &parsed_json_object["data"]["b"][0][0].take_string() {
        Some(s) => String::from(s),
        None => String::from("--None--"),
    };
    // .expect("Error unwraping the parsed json object from the given key");

    let b_quantity = match &parsed_json_object["data"]["b"][0][1].take_string() {
        Some(s) => String::from(s),
        None => String::from("--None--"),
    };
    let a_price_level = match &parsed_json_object["data"]["a"][0][0].take_string() {
        Some(s) => String::from(s),
        None => String::from("--None--"),
    };
    let a_quantity = match &parsed_json_object["data"]["a"][0][1].take_string() {
        Some(s) => String::from(s),
        None => String::from("--None--"),
    };

    let event_time = match &parsed_json_object["data"]["E"].as_u64() {
        Some(s) => *s,
        None => 0,
    };
    // println!("{}", event_time);
    Coin {
        Stream: String::from(stream),
        Symbol: String::from(symbol),
        EventType: String::from(event_type),
        BidPriceLevel: String::from(b_price_level),
        BidQuantity: String::from(b_quantity),
        AsksPriceLevel: String::from(a_price_level),
        AsksQuantity: String::from(a_quantity),
        EventTime: event_time,
    }
}
#[derive(Debug, Clone)]
pub struct Coin {
    pub Stream: String,
    pub Symbol: String,
    pub EventType: String,
    pub BidPriceLevel: String,
    pub BidQuantity: String,
    pub AsksPriceLevel: String,
    pub AsksQuantity: String,
    pub EventTime: u64,
}
impl Coin {
    pub fn get_nul_values() -> Self {
        Self {
            Stream: String::from("Null"),
            Symbol: String::from("Null"),
            EventType: String::from("Null"),
            BidPriceLevel: String::from("Null"),
            BidQuantity: String::from("Null"),
            AsksPriceLevel: String::from("Null"),
            AsksQuantity: String::from("Null"),
            EventTime: 0,
        }
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    Coin_details(String),
    TimeKeeper(Instant),
    SplitButton(button::State, String),
}
pub enum State {
    Idle,
    Fetching { last_tick: Instant },
}
