


pub mod websocket_client;
pub mod websocket_server;
pub mod reliable_message_subsystem;
pub mod websocket_messages;
pub mod util;

use std::sync::Arc;

use crate::websocket_server::WebsocketServer;
use util::websocket_channel::TokioMpscChannel;
use websocket_messages::{MessageReliability, MessageReliabilityType};

use crate::websocket_messages::{InboundMessage, OutboundMessage, OutboundMessageDestination, SocketMessage};
use crate::websocket_client::WebsocketClient;
use tokio::time::Duration;

use serde::{Serialize,Deserialize};


// ----

// You can define your own inner messages to send through the socket server.  They just need to implement Serialize/Deserialize and MessageReliability as follows: 

#[derive(Serialize,Deserialize)]
struct MyCustomMessage {
    
    color: String
}

impl MessageReliability for MyCustomMessage {
    
         fn get_reliability_type(&self, msg_uuid:String) -> MessageReliabilityType{
             return MessageReliabilityType::Reliable( msg_uuid )
         }
}

// -----



#[tokio::main]
async fn main() {
   
    let rt = Arc::new( tokio::runtime::Runtime::new().unwrap() );

    // no ws:// here 
    let server_url:String = "localhost:8100".to_string();

    let mut websocket_server =  WebsocketServer::new() ;

    let mut websocket_server_rx = websocket_server.take_recv_channel().unwrap(); 
    let websocket_server_tx = websocket_server.get_send_channel().clone();
    

    let server_url_clone = server_url.clone();
    let websocket_server_handle = rt.spawn( async move  { 
        websocket_server. start(Some(server_url_clone)).await;

        
    });

     

    println!("Started websocket server");

    let listen_handle = rt.spawn( async move  {  

        loop {
            let recv_msg_result  = websocket_server_rx.recv().await;
       
            if let Some(recv_msg) = recv_msg_result {
                   println!("server received a message! {}" ,recv_msg );
                
            }
         
        }
      
    }); 


    //need ws here 
    let remote_server_url:String = "localhost:8100".to_string();
    
    let base_runtime = Arc::clone(&rt);
    let inner_runtime = Arc::clone(&base_runtime);
    let recv_runtime = Arc::clone(&base_runtime);
    
    base_runtime.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                 
          //boot a client 
                
        let mut socket_conn_recv_channel: TokioMpscChannel<InboundMessage>  = TokioMpscChannel::new(500)  ; 
                                    
        let mut socket_conn_recv_rx = socket_conn_recv_channel.rx.take().unwrap();
    
         let mut  socket_client = WebsocketClient::new();  
         let socket_conn_recv_tx_for_thread = socket_conn_recv_channel.tx.clone();

         
         let socket_connection_result 
         = WebsocketClient::connect(  remote_server_url ).await;
         
                                      

        if let Ok(socket_connection) = socket_connection_result {

            let  socket_conn_send_tx =  socket_connection.get_outbound_messages_tx().await;
        
                println!("client listening");
           
          
               
           
           


          
            //start the client listening thread -- this will await forever  in this spawned rt 
               inner_runtime.spawn(async move {
                    let client_listen_thread= socket_connection. listen( 
                                
                            socket_conn_recv_tx_for_thread.clone()
                        ).await;  
                });
                
                
                
                //send messages from client to server !! 
                //when server sends msgs, they will fill up 'socket_conn_recv_rx' buffer so we can check that buffer for them :) 
                
           let destination = websocket_messages::SocketMessageDestination::All;
            
           let socket_message_result = SocketMessage ::create(  destination, MyCustomMessage{color:"blue".to_string()}); 
           
           if let Ok(socket_message) = socket_message_result {
                socket_conn_send_tx.send(  socket_message ).await ;
           }else{
               println!("could not create message");
           }
            
            
            recv_runtime.spawn(async move {
                loop {
                        if let Some(inbound_msg) = socket_conn_recv_rx.recv().await {
                            println!( "client got a message from the server {}" , inbound_msg );
                            
                        }
                         
                }
            });
            
        }

   
    });
    



    loop {
         //keep this main thread live 
         tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
 