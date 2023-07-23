
use rand::Rng;
use rand::distributions::{Alphanumeric,DistString};

//use the uuid crate 
 use uuid::Uuid;



 //DEPRECATED
pub fn generate_random_hex_string(length: usize) -> String {
    
    //Alphanumeric.sample_string(&mut rand::thread_rng(), length)

    //use the uuid crate
    let uuid = Uuid::new_v4();
    let uuid_string = uuid.to_string();
    //let uuid_string = uuid_string.replace("-", "");
    //let uuid_string = uuid_string[0..length].to_string();
    uuid_string
}





pub fn generate_random_uuid() -> String {
    
    //Alphanumeric.sample_string(&mut rand::thread_rng(), length)

    //use the uuid crate
    let uuid = Uuid::new_v4();
    let uuid_string = uuid.to_string();
    //let uuid_string = uuid_string.replace("-", "");
    //let uuid_string = uuid_string[0..length].to_string();
    uuid_string
}


