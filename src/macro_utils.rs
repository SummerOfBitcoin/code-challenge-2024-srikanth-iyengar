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

#[macro_export]
macro_rules! debug_hex {
    ($vec: expr) => {
        let hex_str: String = $vec.iter().map(|val| format!("{:02x}", *val)).collect();
        eprintln!("{} {:?}", stringify!($vec), hex_str);
    };
}

#[macro_export]
macro_rules! hex_str {
    ($vec: expr) => {{
        let hex_str: String = $vec.iter().map(|val| format!("{:02x}", *val)).collect();
        { hex_str }
    }};
}
