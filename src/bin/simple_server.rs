use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self,AsyncBufReadExt, AsyncWriteExt, BufStream};
use serde_json::{ Value,json};
use tokio::sync::mpsc::{Sender,Receiver};
use tokio::sync::mpsc;
use tokio::net::tcp::ReadHalf;
use serde_derive::{Serialize, Deserialize};
/*
The type Arc<T> provides shared ownership of a value of type T, 
allocated in the heap. Invoking clone on Arc produces a new Arc instance, 
which points to the same allocation on the heap as the source Arc, 
while increasing a reference count. When the last Arc pointer to a given allocation is destroyed, 
the value stored in that allocation (often referred to as “inner value”) is also dropped.
*/
// creating a type alias for user to socket map 
// Arc points top 
type UserToSocket = Arc<Mutex<HashMap<String,mpsc::Sender<String>>>>;

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("127.0.0.1:9090").await;

    // creating a threadsafe hashmap mutex
    let local_db: UserToSocket = Arc::new(Mutex::new(HashMap::new()));

   let listener = match listener{
        Ok(value) => {value},
        Err(_)=> {panic!("ERROR OCCURED")},
    };

    println!("[+] Listener has been started");

    loop {
        // now waiting for connection
        println!("[+] Listening for connection");
        let (mut socket,addr) = listener.accept().await.unwrap();
        println!("[+] A connection accepted from {:?}, spawwning a new task for it",addr);

        // cloning does not actually clone, but rather just increases counter to it
        let ld = Arc::clone(&local_db);

        // spawning a new task
        tokio::spawn(
            async move {
                handler(socket,ld).await;
            }
        );
    }


}

#[derive(Serialize,Deserialize)]
struct msg {
    from: String,
    to: String,
    message: String,
}

impl Default for msg {
    fn default() -> msg {
        msg { 
            from: String::from(""), 
            to: String::from(""), 
            message: String::from("") 
        }
    }
}
//  a handler for new connection
async fn handler(socket: TcpStream, db: UserToSocket) {
    
    // creating a buffered stream
    let mut socket = BufStream::new(socket); 

    // sending banner
    println!("[+] Sending some data to client");
    socket.write_all(b"msg\n").await.unwrap();
    socket.flush().await.unwrap();

    // user response
    let mut line = vec![];

    // creating channels for sending and receiving messages
    let (mut tx, mut rx): (Sender<String>,Receiver<String>)= mpsc::channel(32);
    // parsed _message from user

    //loop{}

    println!("[+] Now waiting to receive any data");
    loop{
        tokio::select! {
            n = socket.read_until(b'\n',&mut line) => {
                let c = line.clone();
                /* 
                let msg = match String::from_utf8(c) {
                    Ok(v) => {v}
                    Err(e) => {println!("An error occured {:?}",e);continue;}
                };*/
                let parsed_value: msg = match serde_json::from_slice(&line) {
                    Ok(v) => {v}
                    Err(e) => {
                        println!("ERROR could not parse json due to error {:?}",e);
                        continue;
                    }
                };

                println!("the user was {}",parsed_value.from);

            }
            msg = rx.recv() => {
                if let Some(vim) = msg {
                    socket.write_all(vim.as_bytes()).await.unwrap();
                }
            }
        }
        
    }

    /*
    // giving some initial info
    socket.write_all(b"[+] Hello Friend, Welcome to my program\r\n").await.unwrap();
    socket.flush().await.unwrap();

    // will hold the response 
    let mut line = vec![0; 1024];

    // holding parsed value
    let mut parsed_value: Value = json!({});
    
    // first lets get the username
    loop {
        // first get username
        if let Err(e)  = socket.read_until(b'\n',&mut line).await {
            println!("Woah error could not get the username {}",e);
            return;
        }

        // parsing json
        let cond = serde_json::from_slice(&line);
        match cond {
            Ok(v) => {
                println!("User sent us a useranme, now parsing it");
                parsed_value = v;
                break
            }
            Err(e) => {
                println!("Got an error, so send again please");
                socket.write_all(b"Please send a valid username").await.unwrap();
            }
        }
    }
   
    line.clear();

    // creating sender and receiver
    let (mut tx, mut rx) = mpsc::channel(10);

    // obtaining username as string
    let username = match &parsed_value["username"]{
        Value::String(v) => {
            v.clone()
        }
        _ => {
            panic!("Undefined type");
        }
    };

    let mut temp = db.lock().unwrap();
    temp.insert(username,tx);*/
}
    
/*
async fn read_from_user(reader: io::ReadHalf<TcpStream>) {
    let mut buf = vec![0;1024];

    loop {
        let n = reader.
        if n == 0 {
            break;
        }
    }
}

async fn write_to_user(writer: io::WriteHalf<TcpStream>) {

}*/
