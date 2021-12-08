use std::io::Write;

use tokio::net::{TcpStream};
use tokio::sync::mpsc::{self, Receiver, Sender};
use user_io::{manage_user_input,handle_server_connection};



#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
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
    
    print!("Please enter your name: ");
    std::io::stdout().flush().unwrap();

    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();

    let i = tokio::spawn(async move {
        handle_server_connection(socket,rx).await;
    });
    let h = tokio::spawn(async move {
        println!("spawnning user input handler");
        manage_user_input(tx,username).await;
    });

    

  
    i.await.unwrap();
    h.await.unwrap();
    
    
}



