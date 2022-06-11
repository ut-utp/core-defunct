
// TODO: docs
#[macro_export]
#[doc(hidden)]
macro_rules! pessimize {
    ($(
        $var_name:ident: $var_ty:ty
    ),* => $fn:ident) => {
        #[inline(never)]
        #[no_mangle]
        fn $fn(
            $(
                $var_name: $var_ty,
            )*
        ) -> ($($var_ty,)*) {
            ($(
                unsafe { ::core::ptr::read_volatile(
                    &$var_name as _
                ) },
            )*)
        }

        #[allow(unused)]
        let ($(
            $var_name,
        )*) = $fn($(
            $var_name,
        )*);
    };
}

pub use crate::pessimize;
