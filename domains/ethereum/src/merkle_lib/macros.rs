#[cfg(feature = "no-sp1")]
#[macro_export]
macro_rules! encode {
    ($out:ident, $e:expr) => {
        $e.encode($out);
        {
            let mut vec = vec![];
            $e.encode(&mut vec);
        }
    };
    ($out:ident, $e:expr, $($others:expr),+) => {
        {
            encode!($out, $e);
            encode!($out, $($others),+);
        }
    };
}
