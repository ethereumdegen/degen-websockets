


use futures_util::StreamExt;
use futures_util::stream::SplitSink;
use futures_util::future::join_all;
 
  //use crossbeam_channel::{ unbounded, Receiver, Sender};
 use tokio::sync::mpsc::{channel, Sender, Receiver};
 
use serde::Serialize;
use serde_json;
 
use std::collections::HashMap;

 
use std::sync::Arc; 
use tokio::sync::{RwLock,Mutex};

use tokio::time::{interval,Duration};


use super::websocket_messages::{
    SocketMessage,
    SocketMessageDestination,
    InboundMessage,
    OutboundMessage,
    
    SocketMessageError,
    
    MessageReliability,
    MessageReliabilityType,
    OutboundMessageDestination
    
    };
    
    
pub struct ReliableMessageSubsystem {
    
    
    
    
    
    
    
    
}


impl ReliableMessageSubsystem {
    
     

//every 0.5 seconds, trysend all the pending reliable messages
pub async fn resend_reliable_messages<T:Clone>(
    pending_reliable_messages: Arc<RwLock<HashMap<String, T>>>,
    global_send_tx: Sender<T> 
) -> std::io::Result<()>  {
    
    let mut interval = interval(Duration::from_secs_f32(0.5));
              
       
     loop {
         interval.tick().await;
         
        let messages = pending_reliable_messages.read().await ;
        for (_, message) in messages.iter() {
            if let Err(e) = global_send_tx.try_send(message.clone()) {  //should be try send !!! 
                eprintln!("Failed to resend reliable message: {}", e);
            }
        }
        
    
     }
    // Ok(())
}
    
    
    
}