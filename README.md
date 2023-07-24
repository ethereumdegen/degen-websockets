## Degen Websockets

A websocket server and client framework supporting: 

* Reliable messaging (option) 
* Broadcast-to-all, Specific-destination or Room-based destinations for messages


### Install 

cargo add degen-websockets


### Examples

See main.rs 

 ### Custom messages 
 
  
  You can define your own inner messages to send through the socket server.  They just need to implement Serialize/Deserialize and MessageReliability as follows: 


```
#[derive(Serialize,Deserialize)]
struct MyCustomMessage {
    
    color: String
}

impl MessageReliability for MyCustomMessage {
    
         fn get_reliability_type(&self, msg_uuid:String) -> MessageReliabilityType{
             return MessageReliabilityType::Reliable( msg_uuid )
         }
}
 

```
