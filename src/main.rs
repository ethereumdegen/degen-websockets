


pub mod websocket_client;
pub mod websocket_server;
pub mod reliable_message_subsystem;
pub mod websocket_messages;
pub mod util;

use crate::websocket_server::WebsocketServer;
use util::websocket_channel::TokioMpscChannel;

use crate::websocket_messages::InboundMessage;
use crate::websocket_client::WebsocketClient;


fn main() {
   
    let rt = tokio::runtime::Runtime::new().unwrap();

    let server_url:String = "localhost:8100".to_string();

    let mut websocket_server =  WebsocketServer::new() ;

    let mut websocket_server_rx = websocket_server.take_recv_channel().unwrap(); 
    let websocket_server_tx = websocket_server.get_send_channel().clone();
    

    let server_url_clone = server_url.clone();
    let websocket_server_handle = rt.spawn( async move  { 
        websocket_server. start(Some(server_url_clone)).await;

        loop{}
    });

     

    println!("Started websocket server");

    let listen_handle = rt.spawn( async move  {  

        loop {
            let recv_msg_result  = websocket_server_rx.recv().await;
       
       
            println!("received a message!");
        }
      
    }); 


    let server_url_clone = server_url.clone();
    rt.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                 
          //boot a client 
                
        let mut socket_conn_recv_channel: TokioMpscChannel<InboundMessage>  = TokioMpscChannel::new(500)  ; 
                                    
        let socket_conn_recv_rx = socket_conn_recv_channel.rx.take().unwrap();
    
         let mut  socket_client = WebsocketClient::new();  
         let socket_conn_recv_tx_for_thread = socket_conn_recv_channel.tx.clone();

         
         let socket_connection_result 
         = WebsocketClient::connect(  server_url_clone ).await;
         
                                      

        if let Ok(socket_connection) = socket_connection_result {

            let  socket_conn_send_tx =  socket_connection.get_outbound_messages_tx().await;
        
                println!("client listening");
            let client_listen_thread= socket_connection. listen( 
                 
               socket_conn_recv_tx_for_thread.clone()
           ).await;  
        }

   
    });
    



    loop {
         //keep this main thread live 
    }
}
 