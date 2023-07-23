
 
use serde::{Serialize,Deserialize};
use tokio_tungstenite::tungstenite::Message;

use crate::net::message_types::{  ClientMessage, ServerMessage, InternalServerMessage };
use crate::websockets::websocket_messages::{MessageReliability};

use super::websocket_messages::MessageReliabilityType;
use thiserror::Error;



 




#[derive(Serialize, Deserialize,Debug ,Clone)]
#[serde(tag = "msg_type", content = "data")]
pub enum CoreDataMessageContents {

    //make sure these uuids cant be cheated / spoofed !?   may need to change architecture ? s
  //UnconnectedClientMsg{ socket_connection_uuid:String, client_message: ClientMessage }, //like for client register, the relay is going to have to await a response and feed it back to client 
  //ConnectedClientMsg{ client_uuid: String,  client_message: ClientMessage} ,  //for when client is already connected 
  ClientMsg(ClientMessage),
  ServerMsg(ServerMessage),
  InternalMsg(InternalServerMessage),
  

}


impl CoreDataMessageContents {
     

   pub fn from_inner_content( inner_content_option: Option<serde_json::Value >) -> Result<Option<Self>, serde_json::Error> {

        
        match inner_content_option {
                Some(inner_content) => {
                     let message:Self = serde_json::from_value( inner_content )?;
                      Ok(Some(message))
                 },
                None => Ok(None) 
        } 
       

     
    }


}




impl MessageReliability for CoreDataMessageContents {
    fn get_reliability_type(&self, message_uuid:String ) -> MessageReliabilityType {  
         
    
        match self {
            
            
            CoreDataMessageContents::InternalMsg( .. ) => MessageReliabilityType::Reliable( message_uuid ),
            
            
            _ => MessageReliabilityType::Unreliable
        }
    
    
    
    }  
}
