
 
use serde::{Serialize,Deserialize};
use tokio_tungstenite::tungstenite::Message;

use crate::net::message_types::{  ClientMessage, ServerMessage, InternalServerMessage };
use crate::websockets::websocket_messages::{MessageReliability};

use super::websocket_messages::MessageReliabilityType;
use thiserror::Error;






/*
//a game logic message 
#[derive(Serialize, Deserialize,Debug ,Clone)] 
pub struct CoreDataMessage { 
   
    pub secure_credentials: Option<SecureMessageCredentials>,  //verified client uuids and stuff.. showing proof we are secure 
    pub contents: CoreDataMessageContents   // the contents and the From info 

}

impl CoreDataMessage {
 
 
    pub fn new( 
       // destination:WrappedMessageDestination,
        contents:CoreDataMessageContents  
        ) ->  Self {

      //  let message_uuid = uuid::Uuid::new_v4().to_string();

        let wrapped_message = Self{
           
            contents,
            secure_credentials: None
        };

        wrapped_message
    }
    
    //typically used by the relay server to tell the server that a message is infact coming from a securely validated client (using the conn registrar)
    pub fn new_with_credentials( 
       // destination:WrappedMessageDestination,
        contents:CoreDataMessageContents,
        secure_credentials: SecureMessageCredentials 
        
        ) ->  Self {

      //  let message_uuid = uuid::Uuid::new_v4().to_string();

        let wrapped_message = Self{
           
            contents,
            secure_credentials: Some(secure_credentials)
        };

        wrapped_message
    }
    
    
    pub fn from_inner_content( inner_content_option: Option<serde_json::Value >) -> Result<Option<Self>, serde_json::Error> {

        
        match inner_content_option {
                Some(inner_content) => {
                     let message:Self = serde_json::from_value( inner_content )?;
                      Ok(Some(message))
                 },
                None => Ok(None) 
        } 
       

       // Ok(message)
    }

    
    pub fn clear_secure_credentials(&mut self) {
        self.secure_credentials = None;
    }
    
     pub fn inject_secure_credentials(
         &mut self, 
         cred: SecureMessageCredentials) 
         {
             
        self.secure_credentials = Some(cred);
    }
    
      pub fn set_secure_credentials(
         &mut self, 
         cred: Option<SecureMessageCredentials>) 
         {
             
        self.secure_credentials = cred;
    }


}

 



impl MessageReliability for CoreDataMessage {
    fn get_reliability_type(&self, message_uuid:String ) -> MessageReliabilityType {  
        
       // let message_uuid = self.
    
    
        match self.contents {
            
            
            CoreDataMessageContents::InternalMsg( .. ) => MessageReliabilityType::Reliable( message_uuid ),
            
            
            _ => MessageReliabilityType::Unreliable
        }
    
    
    
    }  
}

*/





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
    
    //deprecated ? 
  /*  pub fn from_stringified( raw_msg_string:String ) -> Result<Self, serde_json::Error> {

        let message:Self = serde_json::from_str( &raw_msg_string )?;

        Ok(message)
    }*/

   pub fn from_inner_content( inner_content_option: Option<serde_json::Value >) -> Result<Option<Self>, serde_json::Error> {

        
        match inner_content_option {
                Some(inner_content) => {
                     let message:Self = serde_json::from_value( inner_content )?;
                      Ok(Some(message))
                 },
                None => Ok(None) 
        } 
       

       // Ok(message)
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
