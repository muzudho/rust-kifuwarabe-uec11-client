//! Visual studio code [View] - [Terminal]
//!
//! cd C:\Users\むずでょ\source\repos
//! cargo new rust-kifuwarabe-uec11-client
//!
//! cd C:\Users\むずでょ\source\repos\rust-kifuwarabe-uec11-client
//! set RUST_BACKTRACE=full
//! cargo check
//! cargo build
//! cargo run
//!
//! NNGS: admin
//!
//! adminmatch Kifuwarabe Warabemoti b 19 30 0
//!

extern crate serde_derive;
extern crate toml;

mod config;

use config::*;
use encoding_rs::*;
use futures::executor::block_on;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::prelude::*;

async fn connect() {
    match read_toml("./config-uec11.toml".to_string()) {
        Ok(conf) => {
            // v4 か v6 かはサーバー側と合わせること。名前解決はしないので、数で打ち込むこと。
            let host = conf.host.unwrap();
            // Example: "127.0.0.1:3000"
            let host_text = format!("{}:{}", host.domain.unwrap(), host.port.unwrap());
            println!("Trace   | hostText=[{}]", host_text);
            let addr: SocketAddr = match host_text.parse() {
                Ok(x) => x,
                Err(e) => panic!("Error   | socketFail=>{}", e),
            };
            println!("Info    | host=[{}]", &addr);

            let go = conf.game.unwrap();
            println!("Info    | go.");
            let player_number = go.player_number.unwrap();
            println!("Info    | player_number=[{}]", &player_number);
            let player1_name = go.player1_name.unwrap();
            println!("Info    | player1_name=[{}]", &player1_name);
            let player2_name = go.player2_name.unwrap();
            println!("Info    | player2_name=[{}]", &player2_name);
            let first_color = go.first_color.unwrap();
            println!("Info    | first_color =[{}]", &first_color);
            let board_size = go.board_size.unwrap();
            println!("Info    | board_size  =[{}]", &board_size);
            let time_minutes = go.time_minutes.unwrap();
            println!("Info    | time_minutes=[{}]", &time_minutes);
            let seconds_read = go.seconds_read.unwrap();
            println!("Info    | seconds_read=[{}]", &seconds_read);
            let cmd_msec = go.command_interval_msec.unwrap();
            println!("Info    | command_interval_msec=[{}]", &cmd_msec);

            // サーバー・プログラムなら TCPリスナーを作成し、クライアント・プログラムなら TCPストリームを取得する。
            // https://docs.rs/tokio-tcp/0.1.2/src/tokio_tcp/stream.rs.html#49-58
            match TcpStream::connect(&addr).await {
                Ok(mut stream) => {
                    match stream.set_nodelay(true) {
                        Ok(_x) => {}
                        Err(e) => panic!("Error   | nodelayFail=[{}]", e),
                    }

                    println!("Trace   | connectedTo=[{:?}]", stream.peer_addr().unwrap());
                    println!("--------+-------------------");
                    println!("Trace   | We go to the NNGS!");

                    // 最初はサーバーから `Login: ` のメッセージが飛んでくる。
                    show_message_from_server(&mut stream).await;

                    // Interval.
                    println!("Trace   | Please wait for {} ms.", cmd_msec);
                    std::thread::sleep(std::time::Duration::from_millis(cmd_msec));

                    // プレイヤー名を送る。
                    write_message_to_server(
                        &mut stream,
                        match player_number {
                            1 => &player1_name,
                            2 => &player2_name,
                            _ => panic!("Error   | Invalid player number. =[{}]", player_number),
                        },
                    )
                    .await;

                    // Interval.
                    println!("Trace   | Please wait for {} ms.", cmd_msec);
                    std::thread::sleep(std::time::Duration::from_millis(cmd_msec));
                    // マッチ・コマンドを送る。
                    write_message_to_server(
                        &mut stream,
                        &format!(
                            "adminmatch {} {} {} {} {} {}",
                            player1_name,
                            player2_name,
                            first_color,
                            board_size,
                            time_minutes,
                            seconds_read
                        ),
                    )
                    .await;

                    // `Use <match kifuwarabe W 19 30 0> or <decline kifuwarabe> to respond.`
                    // といったメッセージが送られてくるのを待つ。
                    show_message_from_server(&mut stream).await;

                    // `match kifuwarabe W 19 30 0`
                    // といったメッセージを送る。

                    // 対局がついていれば盤面が送られてくるので、それまで待つ。

                    loop {
                        // 標準入力。
                        println!("Trace   | Please key typing.");
                        let mut line = String::new();
                        std::io::stdin().read_line(&mut line).ok();
                        // 改行を削る。
                        line = line.replace("\r", "\n").replace("\n", "");
                        println!("Trace   | input=[{}]", line);

                        match line.as_str() {
                            "exit" => {
                                // ループから抜けます。このコマンドはサーバー側には送れません。
                                break;
                            }
                            "r" => {
                                // `r` コマンドで、サーバーのメッセージ読取。
                                // この状態からは途中で抜けれません。
                                println!("Trace   | Waiting for read.");
                                show_message_from_server(&mut stream).await;
                            }
                            _ => {
                                // その他は、サーバーへのメッセージ送信。
                                write_message_to_server(&mut stream, &line).await;
                            }
                        }
                    }
                }
                Err(e) => println!("Error   | connectFail=[{:?}]", e),
            };
        }
        Err(e) => panic!("Error   | configFail=[{}]", e),
    }
}

async fn write_message_to_server(stream: &mut TcpStream, line: &str) {
    println!("Trace   | write=>{}", line);
    // 改行を付けてください。受信側が 受信完了するために必要です。
    match stream.write_all(format!("{}\n", line).as_bytes()).await {
        Ok(_x) => {
            stream.flush();
            println!("Trace   | Writed.");
        }
        Err(e) => panic!("Error   | writeFail=>{}", e),
    }
}

async fn show_message_from_server(stream: &mut TcpStream) {
    // Read.
    // https://docs.rs/tokio/0.1.12/tokio/prelude/trait.Read.html#tymethod.read
    let mut buffer = [0; 2048];
    // 末尾の改行をもって受信完了。
    match stream.read(&mut buffer[..]).await {
        Ok(size) => {
            // バイト・サイズ表示。
            // println!("Trace   | readSize=[{}]", size);

            // バイナリ表示。
            // println!("Trace   | read=[{:?}]", &buffer[..size]);

            // サーバーから送られてくるのは Shift-JIS かも知れない。変換する。
            let (cow, _encoding_used, _had_errors) = SHIFT_JIS.decode(&buffer[..size]);
            let utf8_text = format!("{}", &cow[..]);
            println!("Trace   | read=[{}]", utf8_text);
            stream.flush();
        }
        Err(e) => panic!("Error   | readFail=>{}", e),
    }
}

#[tokio::main]
async fn main() {
    // syncronized.
    block_on(connect());
    // asyncronized.
    // tokio::spawn(connect());

    // Sleep test.
    println!("Trace   | Please wait 1 seconds.");
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Trace   | Finished.");
}
