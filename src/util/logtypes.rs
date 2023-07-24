
use degen_logger;


pub enum CustomLogStyle {
    
    Warn,
    Error,
    Info,
    Hidden
    
} 


impl degen_logger::DegenLogStyle for CustomLogStyle {
    
    fn bold(&self) -> bool {
        match self {
            Self::Warn => true ,
            Self::Error => false ,
            Self::Info => false, 
            Self::Hidden => false 
        }
    }
    
     fn show(&self) ->  bool {
        match self {
            Self::Warn => false ,
            Self::Error => true ,
            Self::Info => false, 
            Self::Hidden => false 
        }
    }
    
    fn get_log_color( &self ) -> degen_logger::LogColor {
       
         
          match self {
            Self::Warn =>   degen_logger::LogColor::Yellow,
            Self::Info =>   degen_logger::LogColor::Blue,
            Self::Error =>   degen_logger::LogColor::Red ,
            Self::Hidden =>   degen_logger::LogColor::Black 
        }
     }
    
}
