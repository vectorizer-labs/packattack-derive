#![feature(arbitrary_enum_discriminant)]

#[macro_use] extern crate macro_attr;

mod macro_test;

pub trait FromBytes
{
    fn read_from_bytes(bytes: &[u8]) -> Option<&Self>;
}

impl FromBytes for u8
{
    fn read_from_bytes(bytes: &[u8]) -> Option<&Self>
    {
        return bytes.first();
    }
}

// Define macros which derive implementations of these macros.
macro_rules! FromBytes {
    // We can support any kind of item we want.
    (($repr:ty)) => { $repr.read_from_bytes(&bytes); };
    ($($name:ident $( ($($tail:ty),* ) )* = $count:expr),* ) => 
    {
        $($count:expr),* => { Packet::CONNACK },
        //$($name:ident $( ($($tail:ty),* ) )*),*
    };
    (($repr:ty) $(pub)* enum $src_name:ident { $($tail:tt)* } ) => 
    { 
        
        
        impl FromBytes for $src_name
        {
            fn read_from_bytes(bytes: &[u8]) -> Option<& $src_name> 
            {
                match <$repr>::read_from_bytes(bytes)
                {
                    FromBytes!{ $($tail)* }

                    _ => { panic!("Couldn't read {} ", $src_name) }
                }
            }
        }
        
    };  
     
}

macro_attr! {
    #[allow(dead_code)]
    #[derive(Clone, Copy, Debug, FromBytes!(u8))]
    #[repr(u8)]
    pub enum Packet 
    {
        //CONNECT(Protocol, ProtocolLevel, ConnectFlags, KeepAlive),
        CONNACK = 1,
        PUBLISH = 2 ,
        PUBACK = 3,
        PUBREC = 4,
        PUBREL = 5,
        PUBCOMP = 6,
        SUBSCRIBE = 7,
        SUBACK(u8,u8) = 8,
        UNSUBSCRIBE = 9,
        UNSUBACK = 10,
        PINGREQ = 11,
        PINGRESP = 12,
        DISCONNECT = 13,
        AUTH = 14
    }
}