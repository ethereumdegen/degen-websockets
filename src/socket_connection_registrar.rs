

use serde::{Serialize, Deserialize,Deserializer};
use serde::de::DeserializeOwned;
use serde_json;

/*

The relay server needs to use this in order to register clients to understand their uuids!! 
maybe it can somehow inject them into messages being sent to server...


The galaxy server needs to use this in order to register the relay server as the relay server so it can send data TO the relay server (like for init game session)

The grid server needs to use this in order to register the relay server so it can send it the Tick packets !

(in both of the last two cases, the relay server will have to send an Init packet (with a secret for security) to reg itself on the remote server)

*/

use std::collections::HashMap;


#[derive(Serialize,Deserialize,Clone,Debug, Eq, Hash, PartialEq)]
pub enum InternalServer {
    Galaxy,
    Grid, //can have id ? 
    Relay,//can have id ? 
    Star, //can have id ? 
  
  }
  
  
impl std::fmt::Display for InternalServer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InternalServer::Galaxy => write!(f, "Galaxy"),
            InternalServer::Grid  => write!(f, "Grid "),
            InternalServer::Relay  => write!(f, "Relay "),
            InternalServer::Star => write!(f, "Star "),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SocketConnectionRegistrarError {
    NotConnected,
    InvalidDescriptor,
    // More error variants as needed...
}

impl std::fmt::Display for SocketConnectionRegistrarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred in SocketConnectionRegistrar")
    }
}

impl std::error::Error for SocketConnectionRegistrarError {}


#[derive(Serialize,Deserialize,Clone,Debug)]
pub enum SocketConnectionDescriptor { 
    UnconnectedClient,
    ConnectedClient {client_uuid:String},
    Server{ 
        internal_server_uuid:String, 
        server_type: InternalServer, 
        ext_connection_url: String 
    } //server_type,external connection url 
}

impl SocketConnectionDescriptor {
    pub fn get_internal_server_type(&self) -> Result<&InternalServer, SocketConnectionRegistrarError> {
         match self {
             SocketConnectionDescriptor::Server { server_type,.. }  => { Ok(server_type) },
             _ => Err(  SocketConnectionRegistrarError::InvalidDescriptor )
         }
    }
    pub fn get_ext_connection_url(&self) -> Result<&String, SocketConnectionRegistrarError> {
         match self {
             SocketConnectionDescriptor::Server { ext_connection_url, .. }  => { Ok(ext_connection_url) },
             _ => Err(  SocketConnectionRegistrarError::InvalidDescriptor )
         }
    }
    pub fn get_internal_server_uuid(&self) -> Result<&String, SocketConnectionRegistrarError> {
         match self {
             SocketConnectionDescriptor::Server { internal_server_uuid, .. }  => { Ok(internal_server_uuid) },
             _ => Err(  SocketConnectionRegistrarError::InvalidDescriptor )
         }
    }
}

impl std::fmt::Display for SocketConnectionDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SocketConnectionDescriptor::UnconnectedClient => {
                write!(f, "UnconnectedClient")
            }
            SocketConnectionDescriptor::ConnectedClient { client_uuid } => {
                write!(f, "ConnectedClient ({})", client_uuid)
            }
            SocketConnectionDescriptor::Server { server_type, internal_server_uuid, .. }  => {
                write!(f, "Server {} ({})", server_type, internal_server_uuid)
            }
        }
    }
}

pub struct SocketConnectionRegistrar { 
   
    pub connection_map: HashMap<String, SocketConnectionDescriptor>  //socket conn uuid, descriptor 
    
}

impl SocketConnectionRegistrar {
    
    pub fn new() -> Self {
        Self {
            connection_map: HashMap::new()
        }
    }
    
    pub fn register_connection_descriptor( 
        &mut self,
        socket_conn_uuid:String,
        descriptor: SocketConnectionDescriptor
     ){
        self.connection_map.insert(socket_conn_uuid, descriptor);
    }
    
    
    
}