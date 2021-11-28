use tokio::net::{TcpStream};
use tokio::io::{ AsyncWriteExt};
use tokio::io::{self,AsyncBufReadExt, BufStream,BufReader};
use tokio::sync::mpsc::{self, Receiver, Sender};
use serde_derive::{Serialize,Deserialize};
use user_io::{manage_user_input,handle_server_connection};



#[tokio::main]
async fn main() {

    println!("[+] Attemtpting to connect");

    let socket = TcpStream::connect("127.0.0.1:9090").await;

    let socket = match socket {
        Ok(v) => {
            println!("[+] Successfully connected");
            v
        }
        Err(_) => {
            println!("ERROR could not connect to the server");
            std::process::exit(-1);
        }
    };

    
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(32);
    
    tokio::spawn(async move {
        manage_user_input(tx).await;
    });

    tokio::spawn(async move {
        handle_server_connection(socket,rx).await;
    });
    
}



