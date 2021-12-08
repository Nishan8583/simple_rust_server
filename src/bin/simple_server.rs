use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::net::{TcpListener};
use user_io;




#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {

    let listener = TcpListener::bind("127.0.0.1:9090").await.unwrap();
    println!("[+] Listener has been started");

    // creating a threadsafe hashmap mutex
    let local_db: user_io::UserToSocket = Arc::new(Mutex::new(HashMap::new()));
    

    loop {
        // now waiting for connection
        println!("[+] Listening for connection");

        let (socket,addr) = listener.accept().await.unwrap();
        println!("[+] A connection accepted from {:?}, spawwning a new task for it",addr);

        // cloning does not actually clone, but rather just increases counter to it
        let ld = Arc::clone(&local_db);

        // spawning a new task
        tokio::spawn(
            async move {
                user_io::handler(socket,ld).await;
            }
        );
    }


}

