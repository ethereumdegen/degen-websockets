 

 
use futures_util::StreamExt;
use futures_util::stream::SplitSink;
use futures_util::future::join_all;
 
 
use tokio_tungstenite::WebSocketStream;
 
use serde::Serialize;
use serde_json;

 
use tokio::net::{TcpListener, TcpStream};


use futures::SinkExt; 
use std::collections::HashMap; 


use std::thread; 
 
use std::sync::Arc; 
use tokio::sync::{RwLock,Mutex};
 
use std::collections::HashSet;
 
use tokio::time::{interval,Duration};

use tokio_tungstenite::tungstenite::Message;

use crate::{util::rand::generate_random_uuid, states::star_vm_state::SolarSystemOutputEvent};
 
use super::reliable_message_subsystem::ReliableMessageSubsystem;

 use tokio::sync::mpsc::{channel, Sender, Receiver};
//use crossbeam_channel::{ unbounded, Receiver, Sender, TryRecvError, SendTimeoutError};


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
 
 
type ClientsMap = Arc<RwLock<HashMap<String, ClientConnection>>>;
 

type RoomsMap = Arc<RwLock<HashMap<String, HashSet<String>>>>;
 

type TxSink = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;


type RxSink = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;


/*

May have to restructure 
InboundMessage and Outbound Message 
to make it easier to do Ack / response 
*/



pub enum WebsocketSystemEvent {
    ReceivedMessageAck{ reliable_msg_uuid:String }
    
}

#[derive(Debug)]
pub enum WebsocketServerError {
    SendMessageError,
    SerdeJsonError(String),
    TokioError(String),
    SocketMessageErr(String)
    
    
}
 
 
impl std::fmt::Display for WebsocketServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WebsocketServerError::SendMessageError => write!(f, "Could not send message"),
            WebsocketServerError::SerdeJsonError(error) => write!(f, "Serde json error: {}", error),
            WebsocketServerError::TokioError(error) => write!(f, "Tokio Error: {}", error),
            WebsocketServerError::SocketMessageErr(error)  => write!(f,"SocketMessage Error"),
        }
    }
}
 
 
  impl From<crossbeam_channel::TrySendError<OutboundMessage>> for WebsocketServerError {
        fn from(error: crossbeam_channel::TrySendError<OutboundMessage>) -> Self {
        // Here you need to decide how you want to construct your GalaxyServerError
        WebsocketServerError::SendMessageError//( error )
    }
      
  }
  
  
 
impl From<serde_json::Error> for WebsocketServerError {
    fn from(err: serde_json::Error) -> Self {
        // You may want to customize this to better suit your needs
        WebsocketServerError::SerdeJsonError(format!("Serialization error: {}", err))
    }
}
 
impl From<tokio_tungstenite::tungstenite::Error> for WebsocketServerError {
      fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        WebsocketServerError::TokioError(format!("Tokio error: {}", err))
    }
}

impl From<SocketMessageError> for WebsocketServerError {
     fn from(err: SocketMessageError) -> Self {
        // You may want to customize this to better suit your needs
        WebsocketServerError::SocketMessageErr(format!("SocketMessageError: {}", err))
    }
    
}


impl std::error::Error for WebsocketServerError {}



#[derive(Clone)]
pub struct ClientConnection {
    pub client_socket_uuid: String,
    pub addr: String,
    pub tx_sink: TxSink,
}

impl ClientConnection {

    pub fn new( addr:String, client_tx: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message> ) -> Self{

        Self {  
            client_socket_uuid: generate_random_uuid(),
            addr: addr.clone(), 
            tx_sink: Arc::new(Mutex::new( client_tx )) 
        }


    }

    pub async fn send_message(&self, msg: Message) -> Result<(), tokio_tungstenite::tungstenite::error::Error> {
        self.tx_sink.lock().await.send(msg).await
    } 

}



/*


Need to: 
1. implement a loop that every 0.5 seconds will send out the outbound messages in pending_reliable_messages

2. implement a watcher on incoming msg to see if an ACK coems in which will 'delete' our pending reliable msg record 

*/

pub struct WebsocketServer{

    clients: ClientsMap,

    rooms: RoomsMap, // room name -> Set[client_uuid]


    //let (sender, receiver): (Sender<T>, Receiver<T>) = unbounded();
    global_recv_tx: Sender<InboundMessage>, //passed to each client connection 
    global_recv_rx: Option<Receiver<InboundMessage>>,

    global_send_tx: Sender<OutboundMessage>,
    global_send_rx: Option<Receiver<OutboundMessage>>, 
    
    pending_reliable_messages: Arc<RwLock<HashMap<String,OutboundMessage>>>,
    
    
    ws_server_events_tx: Sender<WebsocketSystemEvent>,
    ws_server_events_rx: Option<Receiver<WebsocketSystemEvent>>, 
 
}

impl WebsocketServer {


    pub fn new() -> Self {


        let (global_recv_tx, global_recv_rx): (Sender<InboundMessage>, Receiver<InboundMessage>) = channel(2500);
     
        let (global_send_tx, global_send_rx): (Sender<OutboundMessage>, Receiver<OutboundMessage>) = channel(2500);
       
    
        let (ws_server_events_tx, ws_server_events_rx): (Sender<WebsocketSystemEvent>, Receiver<WebsocketSystemEvent>) = channel(2500);
        
        
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            global_recv_tx,
            global_recv_rx:Some(global_recv_rx),
            global_send_tx,
            global_send_rx:Some(global_send_rx),
            pending_reliable_messages: Arc::new(RwLock::new(HashMap::new())),
            ws_server_events_tx,
            ws_server_events_rx:Some(ws_server_events_rx)
       
        }
    }
    
  /*  pub fn start_in_thread(&mut self, url: Option<String>) ->
     std::io::Result< std::thread::JoinHandle<()> > {
       

        let clients = Arc::clone(&self.clients); 
        let rooms = Arc::clone(&self.rooms);
        let pending_reliable_messages = Arc::clone(&self.pending_reliable_messages);
        
        let global_recv_channel = self.global_recv_tx.clone();

        let global_send_rx = self.global_send_rx.take().unwrap();
        let global_send_tx = self.global_send_tx.clone();
        
        let ws_server_events_tx = self.ws_server_events_tx.clone();
        let ws_server_events_rx = self.ws_server_events_rx.take().unwrap();
        
        let accept_connections_thread = thread::spawn(move || {  //use a non-tokio thread here 
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {

                    let addr: String = url.unwrap_or_else(|| "127.0.0.1:8080".to_string());
                    // Create the event loop and TCP listener we'll accept connections on.
                    let try_socket = TcpListener::bind(&addr).await;
                    let listener = try_socket.expect("Failed to bind");
                    println!("Listening on: {}", addr);
                        
                    
                         
                     let accept_connections = Self::try_accept_new_connections( 
                         Arc::clone(&clients),
                          listener,
                          global_recv_channel , 
                          global_send_tx.clone(), //for sending ACKs
                          ws_server_events_tx.clone()
                          
                            );
                  
                      let send_outbound_messages = Self::try_send_outbound_messages(  
                        Arc::clone(&clients), 
                        Arc::clone(&rooms),
                        global_send_rx 
                    );
                    
                    let resend_reliable_messages = ReliableMessageSubsystem::resend_reliable_messages(
                        Arc::clone(&pending_reliable_messages),  
                        global_send_tx
                    );
                    
                    let handle_server_events = Self::handle_server_events(
                        ws_server_events_rx.clone(),
                        Arc::clone(&pending_reliable_messages   )                     
                    );

             
                            
                 let accept_conn_handle = tokio::spawn(async {
                    accept_connections.await;
                    eprintln!("WARN: ws_s accept_connections ENDED");
                });
                
                let send_outbound_messages_handle = tokio::spawn(async {
                    send_outbound_messages.await;
                    eprintln!("WARN: ws_s send_outbound_messages ENDED");
                });
                
                let resend_reliable_messages_handle = tokio::spawn(async {
                    resend_reliable_messages.await;
                    eprintln!("WARN: ws_s resend_reliable_messages ENDED");
                });
                
                let handle_server_events_handle = tokio::spawn(async {
                    handle_server_events.await;
                    eprintln!("WARN: ws_s handle_server_events ENDED");
                });
                
                tokio::select! {
                    _ = accept_conn_handle => eprintln!("accept_conn_handle finished"),
                    _ = send_outbound_messages_handle => eprintln!("send_outbound_messages_handle finished"),
                    _ = resend_reliable_messages_handle => eprintln!("resend_reliable_messages_handle finished"),
                    _ = handle_server_events_handle => eprintln!("handle_server_events_handle finished"),
                }
 
                     
                     
                     /*   !todo();  
                     tokio::select!{
                         _ => accept_conn_handle
                    
                         
                     };*/
                     
                     println!("WARN: SOCKET SERVER SHUTDOWN");
                 
            });
        });

        println!("Started websocket server");

 

        Ok( accept_connections_thread )
    }*/

    pub async fn start(&mut self, url:Option<String>) -> std::io::Result<()> {
            
        let clients = Arc::clone(&self.clients); 
        let rooms = Arc::clone(&self.rooms);
        let pending_reliable_messages = Arc::clone(&self.pending_reliable_messages);
     
     
        let global_recv_tx = self.global_recv_tx.clone();

        let global_send_rx = self.global_send_rx.take().unwrap();
        
        let global_send_tx = self.global_send_tx.clone();
        
        let ws_server_events_tx = self.ws_server_events_tx.clone();
        let ws_server_events_rx = self.ws_server_events_rx.take().unwrap();
        
        
        let addr: String = url.unwrap_or_else(|| "127.0.0.1:8080".to_string());
        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        println!("Listening on: {}", addr);
    
        let accept_connections = 
          Self::try_accept_new_connections(  
            Arc::clone(&clients),
             listener,
             global_recv_tx,
             global_send_tx.clone(),
             ws_server_events_tx
               );
               
               
        let send_outbound_messages = Self::try_send_outbound_messages( 
            Arc::clone(&clients) , 
            Arc::clone(&rooms),
            global_send_rx
         );
         
        let resend_reliable_messages = ReliableMessageSubsystem::resend_reliable_messages(
            Arc::clone(&pending_reliable_messages),  
            global_send_tx
        );
         
        let handle_server_events = Self::handle_server_events(
                ws_server_events_rx ,
                Arc::clone(&pending_reliable_messages   )                     
            );

        let accept_conn_future =accept_connections;
        
        let send_outbound_messages_future =  send_outbound_messages;
        
        let resend_reliable_messages_future =  resend_reliable_messages;
        
        let handle_server_events_future =  handle_server_events;
        
        tokio::select! {
            _ = accept_conn_future => eprintln!("accept_conn_handle finished"),
            _ = send_outbound_messages_future => eprintln!("send_outbound_messages_handle finished"),
            _ = resend_reliable_messages_future => eprintln!("resend_reliable_messages_handle finished"),
            _ = handle_server_events_future => eprintln!("handle_server_events_handle finished"),
        }
            
            
        println!("WS WARN: TOKIO SELECT DROPPED");
    
        Ok(())
    }
 

    //recv'd client messages are fed into here 
    pub fn take_recv_channel(&mut self) -> Option<Receiver<InboundMessage>> {
        self.global_recv_rx.take() 
    }

    pub fn get_send_channel(&self) -> Sender<OutboundMessage> {
        self.global_send_tx.clone()
    }
    
    
    
   
    pub async fn send_socket_message(&self,  socket_message: SocketMessage, destination: OutboundMessageDestination)
     -> Result<(), WebsocketServerError> {
         
        let reliability_type = socket_message.clone().reliability_type;
      
       // let raw_string = serde_json::to_string(&message)?;
       // let socket_message = SocketMessage::Text(raw_string);
        
       //  let socket_message = SocketMessage::create(destination, message)?
         
         
         let outbound_message = OutboundMessage { 
             destination, 
             message:socket_message
             };
         
       
        if let MessageReliabilityType::Reliable(msg_uuid) = reliability_type {
             self.pending_reliable_messages.write().await.insert(msg_uuid, outbound_message.clone());
         }
         
          self.send_outbound_message(outbound_message)?;
        
         
        Ok(())
    }
    
    /* pub async fn send_reliability_ack( 
         &mut self, 
         ack_message_uuid:String,
         socket_conn_uuid:String 
         )  -> Result<(), WebsocketServerError>{
             
            let outbound_message = OutboundMessage { 
             destination: OutboundMessageDestination::SocketConn( socket_conn_uuid ), 
             message:  SocketMessage::create_reliability_ack(  ack_message_uuid  )
             };
        
        let send_result =  self.send_outbound_message(
           outbound_message
        );
                
      Ok(())
    }*/
    
 
            /*
    pub fn send_wrapped_message<T: Serialize>(&self,  message: T, destination: SocketMessageDestination)
     -> Result<(), WebsocketServerError> {
        let raw_string = serde_json::to_string(&message)?;
        let socket_message = SocketMessage::Text(raw_string);
        
         let outbound_message = OutboundMessage { 
             destination, 
             message:socket_message
             };
         
        self.send_outbound_message(outbound_message)?;
         
        Ok(())
    }*/
    

    pub fn send_outbound_message(&self, msg:OutboundMessage) 
     -> Result<(), WebsocketServerError> {
 
       /* Self::broadcast( 
            Arc::clone(&self.clients), 
            Arc::clone(&self.rooms), 
            msg 
            ).await*/
            
            
        self.get_send_channel().try_send( msg )  .map_err(|_| WebsocketServerError::SendMessageError) //.map_err(|_|   Err(WebsocketServerError::SendMessageError) )

       
    }
    


//move these ???


async fn get_cloned_clients(clients: &ClientsMap) -> Vec<ClientConnection> {
    let clients_map = clients.read().await;
    clients_map.values().cloned().collect()
}


async fn get_cloned_clients_in_room(clients: &ClientsMap, rooms: &RoomsMap, room_name: String ) -> Vec<ClientConnection>  {

    let client_connection_uuids = Vec::new();


    let rooms = rooms.read().await;

    match rooms.get(&room_name) {
        Some(uuid_set) => {}
        None => {}
    }

    return Self::get_cloned_clients_filtered(clients, client_connection_uuids).await;
}


 async fn get_cloned_clients_filtered(clients: &ClientsMap, client_connection_uuids: Vec<String>  ) -> Vec<ClientConnection> {
    let clients_map = clients.read().await;

    let mut filtered_clients: Vec<ClientConnection> = Vec::new();
    
    for uuid in client_connection_uuids {
        if let Some(client_conn) = clients_map.get(&uuid) {
            filtered_clients.push(client_conn.clone());
        }
    }

    filtered_clients
}


 async fn get_cloned_client_specific(clients: &ClientsMap, socket_connection_uuid: String  ) -> Vec<ClientConnection> {
    let clients_map = clients.read().await;

    let mut filtered_clients: Vec<ClientConnection> = Vec::new();
     
    if let Some(client_conn) = clients_map.get(&socket_connection_uuid) {
        filtered_clients.push(client_conn.clone());
    }


    filtered_clients
}





// type RoomsMap = Arc<RwLock<HashMap<String, HashSet<String>>>>;
 
  /*  pub fn add_client_to_room(&mut self, client_connection_uuid:String, room_name : String ) {

        let rooms = Arc::clone(&self.rooms);

        rooms.write.unwrap().insert(); 


    }

    pub fn remove_client_from_room(){

    }
*/


//need to put a loop AROUND the while let 
async fn handle_server_events( 
    mut ws_event_rx: Receiver<WebsocketSystemEvent>,
    pending_reliable_messages: Arc<RwLock<HashMap<String, OutboundMessage>>>,
) -> std::io::Result<()>  {
    
    loop {
         
    while let Some(evt) = ws_event_rx.recv().await { //pass control abck to executor 
        
          println!("ws server handling server event !!! ");
          
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
    pending_reliable_messages: Arc<RwLock<HashMap<String, OutboundMessage>>>,
    message_uuid: String,
    
) {
    let mut messages = pending_reliable_messages.write().await ;
    messages.remove(&message_uuid) ;
}


//every 0.5 seconds, trysend all the pending reliable messages
/*pub async fn resend_reliable_messages(
    pending_reliable_messages: Arc<RwLock<HashMap<String, OutboundMessage>>>,
    global_send_tx: Sender<OutboundMessage> 
) -> std::io::Result<()>  {
    
    let mut interval = interval(Duration::from_secs_f32(0.5));
              
       
     loop {
         interval.tick().await;
         
        let messages = pending_reliable_messages.read().await ;
        for (_, message) in messages.iter() {
            if let Err(e) = global_send_tx.try_send(message.clone()) {
                eprintln!("Failed to resend reliable message: {}", e);
            }
        }
        
    
     }
    // Ok(())
}*/


pub async fn try_send_outbound_messages( 
    clients_map: ClientsMap, 
    rooms_map: RoomsMap,
    mut global_send_rx: Receiver<OutboundMessage> 
) -> std::io::Result<()> {
 

    loop {      
        
        //await gives control back to executor 
        match global_send_rx.recv().await {
            Some(msg) => {
                     
               // let message = msg;
                let clients_map = Arc::clone(&clients_map);
                let rooms_map = Arc::clone(&rooms_map);

                println!("ws_server: try send outbound message  " );

                Self::broadcast(clients_map, rooms_map, msg).await;


            }
          /*  Err(TryRecvError::Empty) => {
                // No messages available right now, sleep for a short duration
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }*/
            None   =>  {} ,
        }
    }

   // Ok(())

}




    pub async fn add_client_to_room(&self, client_connection_uuid:String, room_name: String ) {
        let mut rooms = self.rooms.write().await;
    
        let room_clients = rooms.entry(room_name).or_insert_with(HashSet::new);
        room_clients.insert(client_connection_uuid);
    }
    
    pub async fn remove_client_from_room(&self, client_connection_uuid:String, room_name: String ) {
        let mut rooms = self.rooms.write().await;
    
        if let Some(room_clients) = rooms.get_mut(&room_name) {
            room_clients.remove(&client_connection_uuid);
            
            // Optionally, you can remove the room if it's now empty
            if room_clients.is_empty() {
                rooms.remove(&room_name);
            }
        }
    }
    
 






pub async fn broadcast( 
    clients_map: ClientsMap, 
    rooms_map:RoomsMap,  
    outbound_message: OutboundMessage
) -> Result<(), WebsocketServerError> {
       
    println!("ws_server broadcasting msg: {} ", outbound_message.message  );

    let socket_message = outbound_message.message;
         

    let client_connections = match outbound_message.destination {

       OutboundMessageDestination::All =>  Self::get_cloned_clients(&clients_map).await,
       OutboundMessageDestination::Room(room_name) => Self::get_cloned_clients_in_room(&clients_map,&rooms_map,room_name).await,
       OutboundMessageDestination::SocketConn(socket_connection_uuid) => Self::get_cloned_client_specific(&clients_map,socket_connection_uuid).await,
      // MessageDestination::ResponseToMsg(msg_uuid) => {},
      // MessageDestination::Server => {}
    };

 
    Self::broadcast_to_connections(client_connections, socket_message).await
}


pub async fn broadcast_to_connections
( connections: Vec<ClientConnection>, socket_message: SocketMessage)
 -> Result<(), WebsocketServerError> 
  {
         

   let message = socket_message.to_message()?;
 
    //Could cause thread lock issue !? 
    let send_futures: Vec<_> = {

        connections 
            .iter()
            .map(|client| {
                let message = message.clone();
                client.send_message(message)
            })
            .collect()

    };

    let results = join_all(send_futures).await;

    for result in results {
        if let Err(err) = result {
            eprintln!("Failed to send a message: {}", err);
            
            return Err(WebsocketServerError::SendMessageError);
        }
    }
    Ok(())
}
 
 
 

pub async fn try_accept_new_connections(
    clients_map: ClientsMap, 
    listener: TcpListener, 
    global_recv_tx: Sender<InboundMessage>, //so we can tell outer process about what we recv'd
    
    global_send_tx: Sender<OutboundMessage>, //so we can send ACK packets if got reliable 

    ws_server_events_tx: Sender<WebsocketSystemEvent>
) -> std::io::Result<()> {
 
                                                  
     while let Ok((stream, _)) = listener.accept().await {  //pass control back to executor
        let clients_map =  Arc::clone(&clients_map);
         let new_client_thread = tokio::spawn(
            Self::accept_connection(
                clients_map, 
                stream, 
                global_recv_tx.clone(),
                
                global_send_tx.clone(),
                ws_server_events_tx.clone()
                
            ) 
            );
            
            //new_client_thread.await;
            
            //HOW TO JOIN ?
    }
    Ok(())
}
 
 

async fn accept_connection(
    clients: ClientsMap, 
    raw_stream: TcpStream,
     global_socket_tx: Sender<InboundMessage>,
     
     outbound_messages_tx: Sender<OutboundMessage> ,//for sending ack packets
     ws_server_events_tx: Sender<WebsocketSystemEvent>
    ) {

    let addr = raw_stream
        .peer_addr()
        .expect("connected streams should have a peer address")
        .to_string();

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (   client_tx, mut client_rx) = ws_stream.split();   //this is how i can read and write to this client 
    

    let new_client_connection = ClientConnection::new( addr.clone(),  client_tx  );
    
    let client_uuid = new_client_connection.client_socket_uuid.clone();
    
    let client_socket_uuid = new_client_connection.client_socket_uuid.clone();

    clients.write().await.insert(
         client_socket_uuid.clone(), 
         new_client_connection 
        );

   

         //in this new thread for the socket connection, recv'd messages are constantly collected
    while let Some(msg) = client_rx.next().await {  //pass control back to executor
        match msg {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    //let data = msg.clone().into_data();
                    //println!("Received a message from {}: {:?}", addr, data);
                    // here you can consume your messages

                    let inbound_msg_result = InboundMessage::from_message(
                        client_uuid.clone(),
                        msg
                    );

                    if let Ok(inbound_msg) = inbound_msg_result {
                      global_socket_tx.try_send( inbound_msg.clone()  );
                      
                      let socket_message = inbound_msg.message;
                      if let MessageReliabilityType::Reliable( msg_uuid ) =  socket_message.reliability_type {
                        let ack_message =  SocketMessage::create_reliability_ack( msg_uuid ) ;
 
                                println!("server learned of reliable msg! ");
 
                        
                              let send_ack_result =  outbound_messages_tx.try_send (
                                        OutboundMessage {
                                            destination: OutboundMessageDestination::SocketConn( client_socket_uuid.clone( )),
                                            message:ack_message.clone()
                                        }
                                    );
                                    
                        match send_ack_result {
                             Ok( .. ) => {println!("Server sent ack {}", ack_message)},
                             Err(e) => println!("{}",e)
                         }     
                      }
                      
                      if let SocketMessageDestination::AckToReliableMsg( reliable_msg_uuid ) = socket_message.destination   {
                         let handle_ack_result =  ws_server_events_tx.try_send( 
                           WebsocketSystemEvent::ReceivedMessageAck { reliable_msg_uuid }   
                          );    
                          
                         match handle_ack_result {
                             Ok( .. ) => {println!("Server handling ack")},
                             Err(e) => println!("{}",e)
                         }                  
                          
                      }                    
                      
                      
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "an error occurred while processing incoming messages for {}: {:?}",
                    client_socket_uuid.clone(),
                    e
                );
                break;
            }
        }
    }

    // Remove the client from the map once it has disconnected.
    clients.write().await.remove(&addr);
}
 


 
   


} 





