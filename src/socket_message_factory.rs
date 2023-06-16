use serde_json;

use super::websocket_messages::{InboundMessage,SocketMessage, SocketMessageDestination};

use super::core_data_message::{  CoreDataMessageContents};
use super::socket_connection_registrar::{
    SocketConnectionRegistrar,
    SocketConnectionDescriptor,
    InternalServer,
    
    };
    
use crate::util::rand::generate_random_uuid;
use crate::net::message_types::InternalServerMessage;

pub struct SocketMessageFactory {
    
    
}

impl SocketMessageFactory {
    
    pub fn generate_server_descriptor(
        server_type:InternalServer,
        ext_connection_url: String
      ) -> Result<SocketMessage , serde_json::Error>  { 
          
       let socket_message = SocketMessage::create( 
            SocketMessageDestination::Server,  
           
                CoreDataMessageContents::InternalMsg(  
                InternalServerMessage::RegisterServerDescriptor(
                       SocketConnectionDescriptor::Server { 
                           internal_server_uuid: generate_random_uuid(),
                           server_type , 
                           ext_connection_url 
                        }
                     ) 
               
            ) 
        );
        
      socket_message
    }
    
}