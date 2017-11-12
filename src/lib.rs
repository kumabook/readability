extern crate hyper;
extern crate hyper_native_tls;
#[macro_use]
extern crate html5ever;
extern crate regex;
extern crate url;

pub mod extractor;
pub mod scorer;
pub mod dom;
pub mod error;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
