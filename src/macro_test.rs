

// Define some traits to be derived.
trait TypeName {
    fn type_name() -> &'static str;
}

trait ReprType {
    type Repr;
}

// Define macros which derive implementations of these macros.
#[allow(unused_macros)]
macro_rules! TypeName {
    // We can support any kind of item we want.
    (() $(pub)* enum $name:ident $($tail:tt)*) => { TypeName! { @impl $name } };
    (() $(pub)* struct $name:ident $($tail:tt)*) => { TypeName! { @impl $name } };

    // Inner rule to cut down on repetition.
    (@impl $name:ident) => {
        impl TypeName for $name {
            fn type_name() -> &'static str { stringify!($name) }
        }
    };
}

#[cfg(test)]
mod tests {
    
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn main()
    {
        let bar = Packet::SUBACK(5,2);
        //let Packet::SUBACK(x, y) = bar;
        //println!("values: {} , {} ",x, y);
        //let v = bar as <Packet as ReprType>::Repr;
        //let msg = format!("{:?}: {}", bar);
        println!("message {:?}", bar);
        
        //assert_eq!(msg, "Bar: B (1)");
    }
}