mod postcard;

pub use self::postcard::PostcardEncode;
pub use self::postcard::PostcardDecode;

pub use ::postcard::ser_flavors::{Cobs, Slice};
