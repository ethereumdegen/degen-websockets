

use serde::{Serialize,Deserialize}; 
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::Message;

use uuid;

use thiserror::Error;

 

//move this to shared?

/*
pub trait MessageUuid { 
   fn get_message_uuid(&self) -> String;
}
*/

#[derive(Serialize, Deserialize,Debug ,Clone)] 
pub enum SecureMessageCredentials{
  //  Client {client_uuid: String}, //client uuid 
    Player {player_uuid: String}
} 


pub trait MessageReliability{ 
   fn get_reliability_type(&self, msg_uuid:String) -> MessageReliabilityType;
}


#[derive(Serialize, Deserialize,Debug ,Clone)] 
pub enum MessageReliabilityType {
    Reliable(String), //msg uuid for ack 
    Unreliable
}



#[derive(Serialize, Deserialize,Debug ,Clone)] 
pub enum SocketMessageDestination {
    All,
    Client(String), //client uuid 
    Room(String), 
    ResponseToMsg(String), //message uuid
    AckToReliableMsg(String), //message uuid 
    Server //EcosystemServer) //server type 
}


#[derive(Debug, Error)]
pub enum SocketMessageError {
    #[error("Serialization Error: {0}")]
    SerializationError(serde_json::Error),
    #[error("Unknown Message Type Error")]
    ParseFromUnknownMessageError,
    // Add more errors as needed
}



#[derive(Serialize, Deserialize,Debug ,Clone)] 
pub struct SocketMessage {
    pub message_uuid: String, //used so we can respond to it 
    pub reliability_type: MessageReliabilityType,
    pub destination: SocketMessageDestination, 
    pub inner_contents: Option<serde_json::Value>,   //this is what is passed to the core bsns logic 
    pub secure_credentials: Option<SecureMessageCredentials>,   //can be added by relay server to attest about a connection 
}

impl SocketMessage {
    
    pub fn create<T: Serialize + MessageReliability>( 
        destination:SocketMessageDestination,
        contents: T  
        ) ->  Result<Self, serde_json::Error> {

        let message_uuid = uuid::Uuid::new_v4().to_string();

        let wrapped_message = Self{
            message_uuid:message_uuid.clone(),
            destination, 
            reliability_type: contents.get_reliability_type(message_uuid.clone()),
            inner_contents: Some(serde_json::to_value(contents)? ),
            secure_credentials: None 
        };

        Ok(wrapped_message)
    }
    
    
    pub fn clone_with_credentials(&self, credentials: Option<SecureMessageCredentials> ) -> Self {
        
        let mut clone = self.clone();
         
        clone.secure_credentials =  credentials ;
         
        clone  
         
          
    }
    
    pub fn create_reliability_ack(ack_msg_uuid:String) 
    -> Self {
        
        Self{
            message_uuid: uuid::Uuid::new_v4().to_string(),
            reliability_type: MessageReliabilityType::Unreliable,
            inner_contents: None   ,
            destination: SocketMessageDestination::AckToReliableMsg( ack_msg_uuid ) ,
            secure_credentials: None      
        }
        
    }
        
   pub fn from_stringified  (
       message: serde_json::Value
    ) -> Result<Self, SocketMessageError> {

        let wrapped_message: Self = serde_json::from_value(message).map_err(SocketMessageError::SerializationError)?;

        Ok(wrapped_message)
    }
    
    
    pub fn from_message(msg: Message) -> Result<Self, SocketMessageError> {
        match msg {
            Message::Text(inner) => {
                let value = serde_json::from_str(&inner).map_err(SocketMessageError::SerializationError)?;
                Self::from_stringified(value)
            },
            Message::Binary(inner) => {
                let string = String::from_utf8(inner).map_err(|_| SocketMessageError::ParseFromUnknownMessageError)?;
                let value = serde_json::from_str(&string).map_err(SocketMessageError::SerializationError)?;
                Self::from_stringified(value)
            },
            _ => Err(SocketMessageError::ParseFromUnknownMessageError),
        }
    }
    
    pub fn to_message(&self) -> Result<Message, SocketMessageError > {
        
        let inner = serde_json::to_string( self ).map_err(SocketMessageError::SerializationError)?;
        
        Ok(Message::Text(inner.to_string()))        
    }
    
    pub fn is_reliable(&self) -> bool {
        
        match self.reliability_type {
            MessageReliabilityType::Reliable( .. ) => true,
            MessageReliabilityType::Unreliable => false
        }
    }
 
    
    
    
}

 /*
impl MessageReliability for SocketMessage {
    fn is_reliable(&self, msg_uuid : String ) -> MessageReliabilityType {  
        self.reliability_type.clone()
    }  
}*/

impl std::fmt::Display for SocketMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.inner_contents.clone() {
            Some(inner) =>   write!(
            f,
            "SocketMessage {{ message_uuid: {}, reliability_type: {:?}, destination: {:?}, inner_contents: {} }}",
            self.message_uuid, 
            self.reliability_type,
            self.destination,
            inner
        ),
            None =>  write!(
            f,
            "SocketMessage {{ message_uuid: {}, reliability_type: {:?}, destination: {:?} }}",
            self.message_uuid, 
            self.reliability_type,
            self.destination 
        )
        }
       
        
    }
}

 
 

#[derive(Serialize, Deserialize,Debug ,Clone)] 
pub enum OutboundMessageDestination {
    All,
    SocketConn(String), //socket connection uuid 
    Room(String), 
}





#[derive(Serialize,Deserialize,Clone)]
pub struct OutboundMessage {

    //pub client_socket_uuid: Option<String>, //could be a room? 
    //pub room: Option<String>,
    pub destination: OutboundMessageDestination,
    pub message: SocketMessage   //could make a custom type similar to tokio tungstenite message 

}
  
impl OutboundMessage {
    pub fn new_to_socket_conn_uuid(
        message:SocketMessage,
        socket_conn_uuid: String
    ) -> Self {
        Self {
            destination: OutboundMessageDestination::SocketConn( socket_conn_uuid  ),
            message
        }
    }
}
  
  
  

#[derive(Serialize,Deserialize,Clone)]
pub struct InboundMessage {

    pub socket_connection_uuid: String,
    //message_uuid: String, ? no. 
    pub message: SocketMessage   //could make a custom type similar to tokio tungstenite message 

}
 
 impl std::fmt::Display for InboundMessage {
     
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InboundMessage: {{ socket_connection_uuid: {}, message: {} }}",
            self.socket_connection_uuid,
            self.message
        )
    }
 }


impl InboundMessage {

    pub fn from_message(socket_connection_uuid:String, msg:Message) -> Result<Self,SocketMessageError> {


        let message = SocketMessage::from_message(msg)?;  //text( msg.clone().into_text().unwrap() ) ;

       Ok( Self{
            socket_connection_uuid,
            message 
        } )
    }

} 

 