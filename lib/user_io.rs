
use tokio::net::{TcpStream};
use tokio::io::{ AsyncWriteExt};

use tokio::io::{self,AsyncBufReadExt, BufStream,BufReader};
use tokio::sync::mpsc::{self, Receiver, Sender};
use serde_derive::{Serialize,Deserialize};
use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::Mutex;
#[derive(Serialize,Deserialize,Debug)]
pub struct MsgBox {
    pub from: String,
    pub to:String,
    pub msg: String,
}

impl Default for MsgBox{
    fn default() -> MsgBox{
        MsgBox{
            from: String::new(),
            to: String::new(),
            msg: String::new(),

        }
    }
}

/*
The type Arc<T> provides shared ownership of a value of type T, 
allocated in the heap. Invoking clone on Arc produces a new Arc instance, 
which points to the same allocation on the heap as the source Arc, 
while increasing a reference count. When the last Arc pointer to a given allocation is destroyed, 
the value stored in that allocation (often referred to as “inner value”) is also dropped.
*/
// creating a type alias for user to socket map 
// Arc points top 
pub type UserToSocket = Arc<Mutex<HashMap<String,mpsc::Sender<MsgBox>>>>;

// if you want mutex help: If you need to hold a mutex guard while you're awaiting, you must use an async-aware version of the `Mutex` type.
// https://github.com/rust-lang/rust/issues/71072
#[derive(Serialize,Deserialize,Debug)]
pub struct InitialInfo {
    pub username: String,
}

pub async fn handle_server_connection(socket: TcpStream, mut rx: Receiver<String>) {
        print!("[+] Server connection handler spawned\n");
        io::stdout().flush().await.unwrap();
        
        // receiving username
        let msg = rx.recv().await.unwrap();
        let mut s = BufStream::new(socket);

        // sending initial username json stuff to server
        let i = InitialInfo{
            username: msg,
        };
    
        // converting to json
        let mut value = serde_json::to_string(&i).unwrap();
        value.push_str("\n");
        s.write_all(value.as_bytes()).await.unwrap();
        s.flush().await.unwrap();
        drop(i);

        let mut buf = vec![0];
        loop {
            tokio::select! {
                // receiving from user_input
                v = rx.recv() => {
                    if let Some(msg) = v {
                        //let msg = msg + "\n";
                        println!("message was reeived {}",msg);
                        let temp_msg = MsgBox{
                            from: String::from("Lmfao"),
                            to: String::from("Lmfao"),
                            msg: msg,
                        };
                        let mut json_v = serde_json::to_vec(&temp_msg).unwrap();
                        json_v.push(b'\n');
                       // let final_b = "test\n".as_bytes();
                       //println!("Message being sent is {}",msg);
                        s.write_all(&json_v).await.unwrap();
                        s.flush().await.unwrap();
                        println!("message sent");
                    } else {
                        println!("[+] Your value was not understood");
                    }
                }

                // receiving from server
                v = s.read_until(b'\n',&mut buf) => {
                    let s = v.unwrap();
                    println!("{:?}",s);
                }   
            };
        }
    
}

pub async fn manage_user_input(tx: Sender<String>, username: String)  {

        println!("manage user_input spawned");
        // taking username from user and sending it to receiver channel section
        let inr = io::stdin();
        let mut reader = BufReader::new(inr);
    
        let mut out = io::stdout();
        
        let mut user_input = vec![0];
       
        let mut u  = username.clone();

        tx.send(username).await.expect("ERROR could not send username to the receiver channel");
        println!("value sent to channel");
    
        
        u.push_str(" > ");
        println!("before loop");

        user_input.clear();
        loop {
            print!("You want to chat with : ");
            out.flush().await.unwrap();

            if let Err(e) =  reader.read_until(b'\n', &mut user_input).await {
                println!("ERROR could not read input for message receiver {:?}",e);
                continue;
            }
    
            let to_b = user_input.clone();
            let to = String::from_utf8(to_b).expect("ERROR occured");
            
            let to_c = to.clone();
            let mut user_msg = MsgBox::default();
            user_msg.to = to.to_owned();
            
            user_input.clear();
    
            let mut prompt=u.clone().replace("\n","");
            prompt.push_str("-");
            prompt.push_str(to_c.as_str());
            prompt.push_str(" > ");
            loop {
                print!("Prompt: {}",prompt);
                out.flush().await.unwrap();
    
                if let Err(err) =  reader.read_until(b'\n', &mut user_input).await {
                    println!("ERROR could not read your message {:?}",err);
                    break;
                }
    
                let msg_b = user_input.clone();
                user_input.clear();
    
                let mut msg = match String::from_utf8(msg_b) {
                    Ok(v) => {
                        v
                    }
                    Err(e) => {
                        println!("ERROR while trying to convert your value to string err {:?}",e);
                        break;
                    }
                };
                msg.pop();
                msg.pop();
                let temp_msg = MsgBox{
                    from: String::from("Lmfao"),
                    to: String::from("Lmfao"),
                    msg: msg.to_owned(),
                };
                //user_msg.msg=msg.to_owned();
                let mut v = serde_json::to_string(&temp_msg).expect("ERROR could not get json value from struct");
                //let mut v = v.replace("\n", "");
                v.push_str("\n");
                println!("final mesasge becomes {}",v);
                tx.send(msg).await.unwrap();
            }
    
        }
    }


pub async fn handler(socket: TcpStream, db: UserToSocket) {
        
    // creating a buffered stream
    let mut socket = BufStream::new(socket); 
    
    // vector will store clients message
    let mut line = vec![];

    // reading the username
    socket.read_until(b'\n',&mut line ).await.unwrap();

    // deserialize json
    let username_struct: InitialInfo =  serde_json::from_slice(&line).unwrap();

    println!("new user {}",username_struct.username);
    let mut temp_db = db.lock().await;
    
    // creating channels
    let (tx, mut rx) = mpsc::channel(10);
    temp_db.insert(username_struct.username,tx);

    // droppping it manually so it is unlocked
    drop(temp_db);


    line.clear();

    let err_msg: MsgBox = MsgBox{
        from:String::from("server"),
        to:String::from("client"),
        msg:String::from("ERROR could not load messages"),
    };
    let err_msg = serde_json::to_vec(&err_msg).unwrap();
    println!("waiting to get data");
    let v = socket.read_until(b'\n',&mut line).await.unwrap();
        println!("message received {}",v);
    println!("before looping");
    loop {
        line.clear();
        tokio::select!{
            // checking if any other user has sent this user some messages
            v = rx.recv() => {
                if let Some(value) = v {
                    if let Ok(mesg) = serde_json::to_vec(&value) {
                        if let Err(e) =  socket.write_all(&mesg).await {
                            println!("ERROR could not send data to client {:?}",e);
                            continue;
                        }
                    } else {
                        if let Err(e) =  socket.write_all(&err_msg).await {
                            println!("ERROR could not send data to client {:?}",e);
                            continue;
                        }
                    }
                    
                }
            }

            // if this user has sent some data
            v = socket.read_until(b'\n',&mut line) => {
                println!("got some value i think");
                match v {
                    Ok(0) => {
                        println!("Client closed the connection");
                        return;
                    }
                    Ok(_) => {
                        let l_c = line.clone();
                        let s = String::from_utf8(l_c).unwrap();
                        println!("temp printing {}",s);
                        let destination_struct: MsgBox = match serde_json::from_slice(&line){
                            Ok(v) => {v}
                            Err(err) => {
                                println!("ERROR could not unmarshall the users message {:?}",err);
                                continue;
                            }
                        };
                        println!("message received {}",destination_struct.msg);
                        let  temp = db.lock().await;
                        if let Some(rx2) = temp.get(destination_struct.to.as_str()) {
                            if let Err(e)= rx2.send(destination_struct).await {
                                println!("ERROR could not send message to another user {:?}",e);
                                continue
                            }
                        };
                        
                        continue;
                    }
                    Err(err) => {
                        println!("ERROR some error occured while reading from client {:?}",err);
                        return;
                    }
                }
            }
        };
    };

    
}