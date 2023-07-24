
 
use serde::{Serialize,Deserialize};
use tokio_tungstenite::tungstenite::Message;

 
use crate::websocket_messages::{MessageReliability};

use super::websocket_messages::MessageReliabilityType;
use thiserror::Error;



 




#[derive(Serialize, Deserialize,Debug ,Clone)]
#[serde(tag = "msg_type", content = "data")]
pub enum CoreDataMessageContents {

  

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
             
            
            _ => MessageReliabilityType::Unreliable
        }
    
    
    
    }  
}
