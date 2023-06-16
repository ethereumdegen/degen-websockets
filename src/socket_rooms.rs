


pub struct Room {
  
  
  
}


impl Room {
  
  pub fn name_from_grid_uuid(grid_uuid:String) -> String{
    
     format!("{}:{}","grid",grid_uuid)
  }
  
  
  pub fn name_from_solar_system_uuid(solar_system_uuid:String) -> String{
     
     format!("{}:{}","solarsystem",solar_system_uuid)
  }
  
}