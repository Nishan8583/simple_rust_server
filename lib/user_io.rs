
use tokio::net::{TcpStream};
use tokio::io::{ AsyncWriteExt};

use tokio::io::{self,AsyncBufReadExt, BufStream,BufReader};
use tokio::sync::mpsc::{self, Receiver, Sender};
use serde_derive::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
struct MsgBox {
    from: String,
    to:String,
    msg: String,
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

#[derive(Serialize,Deserialize)]
pub struct InitialInfo {
    username: String,
}
    pub async fn handle_server_connection(socket: TcpStream, mut rx: Receiver<String>) {
        println!("[+] Server connection handler spawned");
    
        let msg = rx.recv().await.unwrap();
    
        let mut s = BufStream::new(socket);
    
        let i = InitialInfo{
            username: msg,
        };
    
        let value = serde_json::to_string(&i).unwrap();
        s.write_all(value.as_bytes()).await.unwrap();
    
        let mut buf = vec![0];
        loop {
            tokio::select! {
                v = rx.recv() => {
                    if let Some(msg) = v {
                        s.write_all(msg.as_bytes()).await.unwrap();
                    } else {
                        println!("[+] Your value was not understood");
                    }
                }
                v = s.read_until(b'\n',&mut buf) => {
                    println!("{:?}",v);
                }
            };
        }
    
    }

    pub async fn manage_user_input(tx: Sender<String>)  {

        // taking username from user and sending it to receiver channel section
        let inr = io::stdin();
        let mut reader = BufReader::new(inr);
    
        let mut out = io::stdout();
        print!("Please enter your username: ");
        out.flush().await.expect("ERROR could not print output");
    
        let mut user_input = vec![0];
        if let Err(err) = reader.read_until(b'\n',&mut user_input).await {
            println!("Error could not read your input {:?}",err);
            return
        }
    
        let u_bytes = user_input.clone();
        let username = String::from_utf8(u_bytes).expect("ERROR could not convert user input to string");
        let mut u  = username.clone();
        tx.send(username).await.expect("ERROR could not send username to the receiver channel");
    
    
        println!("\n");
        user_input.clear();
    
        u.push_str(" > ");
    
        loop {
            print!("Message to: > ");
            out.flush().await.unwrap();
    
            if let Err(e) =  reader.read_until(b'\n', &mut user_input).await {
                println!("ERROR could not read input for message receiver {:?}",e);
                continue;
            }
    
            let to_b = user_input.clone();
            let to = String::from_utf8(to_b).expect("ERROR occured");
            let to_c = to.clone();
            let mut user_msg = MsgBox::default();
            user_msg.to = to;
            user_input.clear();
    
            let mut prompt=u.clone();
            prompt.push_str("-");
            prompt.push_str(to_c.as_str());
            prompt.push_str(" > ");
            loop {
                print!("{}",prompt);
                out.flush().await.unwrap();
    
                if let Err(err) =  reader.read_until(b'\n', &mut user_input).await {
                    println!("ERROR could not read your message {:?}",err);
                    break;
                }
    
                let msg_b = user_input.clone();
                user_input.clear();
    
                let msg = match String::from_utf8(msg_b) {
                    Ok(v) => {
                        v
                    }
                    Err(e) => {
                        println!("ERROR while trying to convert your value to string err {:?}",e);
                        break;
                    }
                };
    
                user_msg.msg=msg;
                let v = serde_json::to_string(&user_msg).expect("ERROR could not get json value from struct");
                tx.send(v).await.unwrap();
            }
    
        }
    }