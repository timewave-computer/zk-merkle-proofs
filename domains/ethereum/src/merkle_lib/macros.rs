#[cfg(feature = "web")]
#[macro_export]
macro_rules! encode {
    ($out:ident, $e:expr) => {
        $e.encode($out);
        {
            let mut vec = vec![];
            $e.encode(&mut vec);
            println!("{}: {:?}", stringify!($e), vec);
        }
    };
    ($out:ident, $e:expr, $($others:expr),+) => {
        {
            encode!($out, $e);
            encode!($out, $($others),+);
        }
    };
}
