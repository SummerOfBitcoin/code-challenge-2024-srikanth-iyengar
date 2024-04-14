#[macro_export]
macro_rules! debug {
    // a util macro to debug things because I don't want to write println! and the formatting thing
    // over and over :)
    ($($x: expr), +) => {
        $(
            eprintln!("{}: {:?}", stringify!($x) , $x);
        )+
    };
}
