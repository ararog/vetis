pub mod path;

pub trait VirtualHost {
    fn hostname(&self) -> String;

    fn is_secure(&self) -> bool;

    fn set_paths<P>(&mut self, paths: Vec<P>)
    where
        P: path::Path;
    
    fn paths<P>(&self) -> Vec<&P>
    where
        P: path::Path;
}