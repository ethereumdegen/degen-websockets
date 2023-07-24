//! A simple example of hooking up stdin/stdout to a WebSocket stream.
//!
//! This example will connect to a server specified in the argument list and
//! then forward all data read on stdin to the server, printing out all data
//! received on stdout.
//!
//! Note that this is not currently optimized for performance, especially around
//! buffer management. Rather it's intended to show an example of working with a
//! client.
//!
//! You can use this example together with the `server` example.



use degen_logger;

use futures::Future;
/*

Add options for auto reconnect ? 
add crossbeam channels ? 




May have to build some memory slots which keep track of awaiting threads which are waiting on msg responses/ACKs. 

Bc - need a way to send a message that awaits a response ! 

*/
use serde::{Serialize};
use serde_json;

use futures_util::{ StreamExt, SinkExt}; 
use tokio_tungstenite::{connect_async, tungstenite::Message};
  
//use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream,MaybeTlsStream};
use tokio::net::TcpStream;
 
 //use crossbeam_channel::{ unbounded, Receiver, Sender};
 use tokio::sync::mpsc::{channel, Sender, Receiver};

use std::thread;
use tokio::runtime::Runtime;
 

use crate::util::logtypes::CustomLogStyle;

use super::reliable_message_subsystem::ReliableMessageSubsystem;

 
use std::sync::Arc; 
use tokio::sync::{RwLock,Mutex};
 
use std::collections::HashMap;

use super::websocket_messages::{
    SocketMessage,
    InboundMessage,
     OutboundMessage,
     
   
    MessageReliability, 
    MessageReliabilityType,
    
    SocketMessageError,
    
    SocketMessageDestination, OutboundMessageDestination
    };


use super::websocket_server::WebsocketSystemEvent;

use tokio::time::{interval,Duration};
 
 use std::fmt;


#[derive(Debug)]
pub enum WebsocketClientError {
    UnableToConnect,
    SendMessageError,
    SerdeJsonError(String),
    TokioError(String),
    SocketMessageErr,
    NoConnectionError
    
}
 
 
impl fmt::Display for WebsocketClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebsocketClientError::UnableToConnect => write!(f, "Unable to connect"),
            WebsocketClientError::SendMessageError => write!(f, "Could not send message"),
            WebsocketClientError::SerdeJsonError(error) => write!(f, "Serde json error: {}", error),
            WebsocketClientError::TokioError(error) => write!(f, "Tokio Error: {}", error),
            WebsocketClientError::SocketMessageErr => write!(f,"Socket Message Error"),
            WebsocketClientError::NoConnectionError => write!(f,"No Connnection Error")
        
        }
    }
}
 
impl From<serde_json::Error> for WebsocketClientError {
    fn from(err: serde_json::Error) -> Self {
        // You may want to customize this to better suit your needs
        WebsocketClientError::SerdeJsonError(format!("Serialization error: {}", err))
    }
}
 
impl From<tokio_tungstenite::tungstenite::Error> for WebsocketClientError {
      fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        WebsocketClientError::TokioError(format!("Tokio error: {}", err))
    }
}


impl From<SocketMessageError> for WebsocketClientError {
     fn from(err: SocketMessageError) -> Self {
         WebsocketClientError::SocketMessageErr 
          
    }
    
}

impl std::error::Error for WebsocketClientError {}
  
 
type SocketWriteSink = futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type SocketReadStream = futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
 
 
pub struct ConnectionResources {
    
    pub write: Arc< Mutex<SocketWriteSink> >,
     pub read: Option< SocketReadStream > , //can be used like a one-time mutex !  
    
   
     
 
    outbound_messages_rx: Option<Receiver<SocketMessage>>,
    
    pending_reliable_messages: Arc<RwLock<HashMap<String,SocketMessage>>>,
  
    
     
    ws_server_events_rx: Option<Receiver<WebsocketSystemEvent>>, 
    
     outbound_messages_tx: Sender<SocketMessage>, 
      ws_server_events_tx: Sender<WebsocketSystemEvent>, 
        pub socket_connection_uuid: String,
}
 
 
pub struct Connection { 
 
    resources: Arc<Mutex<ConnectionResources>>,
    
}





impl Connection {
        
           
          
       
    
      pub async fn start_listening(
         
        resources: Arc<Mutex<ConnectionResources>>,
        sender_channel: Sender<InboundMessage>, 
        ) {
            
             let mut resources = resources.lock().await; 
           
             let read = resources.read.take().expect("The read stream has already been consumed.");
             let socket_connection_uuid = resources.socket_connection_uuid.clone();
 
             let pending_reliable_messages = Arc::clone(&resources.pending_reliable_messages);
           
             
             let outbound_messages_tx = resources.outbound_messages_tx.clone();
             let outbound_messages_rx = resources.outbound_messages_rx.take().unwrap();
            
             let ws_server_events_tx = resources.ws_server_events_tx.clone();
             let ws_server_events_rx = resources.ws_server_events_rx.take().unwrap();
             
             let write = Arc::clone(&resources.write);
         
               
               
               //this is nottt working 
              let forward_inbound_msg_future =  Connection::forward_inbound_messages(   
                        read,
                        sender_channel,
                        socket_connection_uuid.clone(),
                        outbound_messages_tx.clone(),  // for sending reliability ack 
                        ws_server_events_tx.clone()
                 ) ;
                 
                 
                let send_outbound_msg_future = Self::start_forwarding_outbound_messages (
                    write,
                    outbound_messages_rx,
                    
                    Arc::clone(&pending_reliable_messages),
                    socket_connection_uuid.clone() 
                    
                );
                 
                let resend_reliable_messages = ReliableMessageSubsystem::resend_reliable_messages (
                        Arc::clone(&pending_reliable_messages),  
                        outbound_messages_tx.clone()
                    );
                    
                let handle_server_events = Self::handle_server_events(
                 ws_server_events_rx ,
                 Arc::clone(&pending_reliable_messages   )                     
                );
                 
                    
                 
                    let select = tokio::select! {
                    _ = forward_inbound_msg_future => eprintln!("forward_inbound_msg_handle finished"),
                    _ = send_outbound_msg_future => eprintln!("send_outbound_msg_handle finished"),
                    _ = resend_reliable_messages => eprintln!("resend_reliable_messages_handle finished"),
                    _ = handle_server_events => eprintln!("handle_server_events_handle finished"),
                   };
                    
                    
                    
                   degen_logger::log(  format!("WS WARN: TOKIO SELECT DROPPED")  , CustomLogStyle::Error  ) ; 
 
              
                 
      
    }
    
     
     
         
          async fn start_forwarding_outbound_messages(
         
            write: Arc<Mutex<SocketWriteSink>>,
            mut receiver_channel: Receiver<SocketMessage>,
            
            pending_reliable_messages: Arc<RwLock<HashMap<String, SocketMessage>>>,
            socket_connection_uuid:String, 
        ) -> std::io::Result<()>     
        {
            
              
                  
            degen_logger::log(  format!("ws client start_forwarding_outbound_messages")  , CustomLogStyle::Info  ) ; 
        
            
            
            loop {
                 
                 
            while let Some(socket_message) =  receiver_channel.recv().await {   //let up so other threads in the join  can run 
                  
                    
                    
                    
                    
                       let reliability_type = socket_message.clone().reliability_type;
         
                        
                        if let MessageReliabilityType::Reliable(msg_uuid) = reliability_type {
                                
 
                            //could cause deadlock !? 
                            pending_reliable_messages.write().await.insert(msg_uuid,   socket_message.clone( ) );
                        }
                                      
                    
                    
                
                     degen_logger::log( format!("ws client is sending out msg: {} ", socket_message)  , CustomLogStyle::Info  ) ; 
 
              
                    
                    
                    
                          
                          
                    let message_result = socket_message.to_message();
                    
                     if let Ok(message) = message_result {
                        let send_msg =  write.lock().await.send( message ).await ;    
                        
                        if let Err(e) = send_msg  {
                            
                                degen_logger::log(  format!("ws client: Error sending message.. {}", e)  , CustomLogStyle::Error  ) ; 
 
                            
                           
                        }    
                     }
                     
                  //  Ok(())
                
            }
            
         
            
            }
        
          //  Ok(())
        }
    
    
    //should be a SocketMessage sender right... oh well anyways 
    pub async fn get_outbound_messages_tx(&self) -> Sender<SocketMessage> {
        
        self.resources.lock().await.outbound_messages_tx.clone()
    }
    
    
     pub async fn listen( &self, sender_channel: Sender<InboundMessage>){
        
          
          let resources = Arc::clone(&self.resources);
          
        
        let start_listening_future=   Connection::start_listening(resources, sender_channel); 
           
        tokio::join!( start_listening_future ) ;
        
        degen_logger::log(  format!("ws client: listen loop ended")  , CustomLogStyle::Error  ) ; 
 
    }
    
     
    
    
    
    pub async fn get_socket_connection_uuid(&self) -> String {
        
        
       return  self.resources.lock().await.socket_connection_uuid.clone( )
        
        
    } 
    

async fn handle_server_events( 
    mut ws_event_rx: Receiver<WebsocketSystemEvent>,
    pending_reliable_messages: Arc<RwLock<HashMap<String, SocketMessage>>>,
) -> std::io::Result<()>  {
    
    loop {
    
        while let Some(evt) = ws_event_rx.recv().await {  //give control back to async executor
            
            match evt {
                
                WebsocketSystemEvent::ReceivedMessageAck { reliable_msg_uuid } => {
                    //pending_reliable_messages.write().await.
                    Self::clear_pending_reliable_message(
                        Arc::clone(&pending_reliable_messages),
                        reliable_msg_uuid
                    ).await;
                }
                
                };
            
            }
        
    }
    
        
    // Ok(())
    }

//when we receive an ACK with this message uuid, we clear 
pub async fn clear_pending_reliable_message( 
    pending_reliable_messages: Arc<RwLock<HashMap<String, SocketMessage>>>,
    message_uuid: String,
    
) {
    let mut messages = pending_reliable_messages.write().await ;
    messages.remove(&message_uuid) ;
}
    
    
    /*
    pub async fn send_message_immediately(&mut self,  socket_message:  SocketMessage ) 
    -> Result<(), WebsocketClientError>
    {  
          self.write.lock().await.send( socket_message.to_message()? ).await?;
        
      Ok(())
    }*/
   
    
    //this should loop forever and never end 
    //this also should not take up 100% of the thread use 
    pub async fn forward_inbound_messages( 
        mut read: futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, 
        sender_channel: Sender<InboundMessage>,
        socket_connection_uuid:String  ,
        outbound_messages_tx: Sender<SocketMessage>,
        ws_server_events_tx: Sender<WebsocketSystemEvent>
            ){
                   
        loop{    
            
              
             
              degen_logger::log(  format!("ws_client client forward_inbound_messages")  , CustomLogStyle::Info  ) ; 
 
           
                        //this await yields back to the executor so its ok to loop ! 
                while let Some(message_result) = read.next().await {
                   
                      match message_result {
                        Ok(message) => {
                            
                            let socket_message_result = SocketMessage::from_message(message) ;
                            
                            if let Ok(socket_message) = socket_message_result {
                                
                                
                             //println!("client got an inbound msg {}", socket_message);
                                
                                let inbound_msg = InboundMessage {
                                    socket_connection_uuid: socket_connection_uuid.clone(),
                                    message:  socket_message.clone(), 
                                };
                                
                                //parse the message into a SocketMessage 
                                
                                //check to see if socket message ie reliable 
        
                                // Send the message into the   channel
                                sender_channel.try_send(inbound_msg.clone()) ;
                                
                                degen_logger::log(   format!("client got socket_message {} ", socket_message) , CustomLogStyle::Info  ) ; 
 
                                
                               
                                if let MessageReliabilityType::Reliable(msg_uuid) = socket_message.reliability_type {
                                    //we need to send an ack ! 
                                    let ack_message =  SocketMessage::create_reliability_ack( msg_uuid.clone()) ;
                                    println!("client creating reliability ack for {}", msg_uuid.clone());
                                    let send_ack_result = outbound_messages_tx.try_send (  //should be try send as to not block 
                                       
                                        ack_message
                                        
                                    );
                                    
                                    if let Err(send_ack_err) = send_ack_result {
                                        eprintln!("send ack error {}", send_ack_err);
                                    }
                                }
                                
                                  if let SocketMessageDestination::AckToReliableMsg( reliable_msg_uuid ) = socket_message.destination   {
                                       
                                    degen_logger::log(   format!("client got ack") , CustomLogStyle::Info  ) ; 
 
                                     
                                     ws_server_events_tx.try_send( 
                                    WebsocketSystemEvent::ReceivedMessageAck { reliable_msg_uuid }   
                                    ) ;                      
                          
                                }      
                             
                             
                            }
                            
                         
                             
                             
                        }
                        Err(e) => {
                            eprintln!("Error while reading message: {:?}", e);
                            break;
                        }
                    }
                }
               
                        
        }
    }
  
    
      
          
          
}



pub struct WebsocketClient{
    pub connection: Option<Connection>,
    
     
}

impl WebsocketClient {

    pub fn new() -> Self {
        
        Self {
             connection: None,
            }
    }
     
  

    pub async fn connect(  connect_addr: String ) 
    -> Result<Connection, WebsocketClientError > {


        let mut final_connect_addr = connect_addr;

        if !final_connect_addr.starts_with("ws://") {
            // append it to the beginning
            final_connect_addr = format!("ws://{}", final_connect_addr);
        }

       
        let url = url::Url::parse(&final_connect_addr).unwrap(); 
       

        for  i in 0..9 {
            match connect_async(url.clone()).await {
                Ok((ws_stream, _)) => {
                   
                     degen_logger::log(   format!("WebSocket handshake has been successfully completed") , CustomLogStyle::Info  ) ; 
 
                    
                    
                    let (write, read) = ws_stream.split();
                    
                    let socket_connection_uuid =  uuid::Uuid::new_v4().to_string();
                    
                    let (outbound_messages_tx, outbound_messages_rx) : (Sender<SocketMessage>, Receiver<SocketMessage>)= channel(500);
                    let (ws_server_events_tx, ws_server_events_rx) : (Sender<WebsocketSystemEvent>, Receiver<WebsocketSystemEvent>)= channel(500);
                    
                    let resources = ConnectionResources {
                        
                        write:Arc::new(Mutex::new(write)),
                        read : Some(read),
                      
                        pending_reliable_messages: Arc::new(RwLock::new(HashMap::new())),
                        outbound_messages_rx: Some(outbound_messages_rx),
                        outbound_messages_tx,
                        
                        ws_server_events_rx: Some(ws_server_events_rx),
                        ws_server_events_tx,
                         socket_connection_uuid,
                    };
                        
                   return Ok( Connection { 
                     
                        
                        resources: Arc::new(Mutex::new(resources)),
                      
                         
                      
                    } );
                    // once connected, break the loop
                    //break;
                },
                Err(e) => {
                    println!("Failed to connect, retrying in 1 second...");
                    // wait for 1 second
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                   
                }
            }
        }
      //  Ok(())
        return Err(  WebsocketClientError::UnableToConnect );
    }
    
     
    
       pub async fn listen_future(
        &mut self,
         conn: Connection,
         channel:  Sender<InboundMessage>  //we send inbound msgs into here 
         )  
       -> impl Future<Output = ()> + '_
        {
         
        self.add_connection( conn );
        
        self.connection.as_mut().unwrap().listen( channel) 
                
        
     }
    
     pub async fn listen(
        &mut self,
         conn: Connection,
         channel:  Sender<InboundMessage>  //we send inbound msgs into here 
         ){
         
        self.add_connection( conn );
        
        self.connection.as_mut().unwrap().listen( channel).await ;
                 
     }
    
   
    
    pub fn add_connection( &mut self ,  connection:  Connection ){
         
         self.connection = Some(connection);
         
    }
    
    
    
    pub async fn get_outbound_messages_tx(&self) -> Result<Sender<SocketMessage>, WebsocketClientError> { 
        
        match &self.connection {
            Some(conn) => Ok ( conn.get_outbound_messages_tx().await ) ,
            None => Err(  WebsocketClientError::NoConnectionError )
        }
       
    }
    
     
    

}
 