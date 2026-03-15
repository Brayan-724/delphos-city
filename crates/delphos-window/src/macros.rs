#[doc(hidden)]
macro_rules! create_delegate {
    ([$name:ident]
     $($tail:tt)*
    ) => {
        $crate::macros::create_delegate!{#macro_rules [$] [$name] $($tail)*}
    };

    (#macro_rules [$D:tt] [$name:ident]
     [$original:path] [$mod:path]
     [$trait:ident $(: $base:path)?] [$ctx:ident]
     data: $datause:tt $data:tt $datac:tt
     impl: $impl:tt
     $(impl-extra: $impl_extra:tt)?
     trait: $trait_extra:tt
    ) => {

        $crate::macros::create_delegate!{#trait
            [$mod]
            [$trait $(: $base)?]
            [$ctx]
            $impl
            $trait_extra
        }

        macro_rules! $name {
            (impl [$D ($D gen:tt)*] for $D State:ty) => {
                ::smithay_client_toolkit::$name!{ @$D ($D gen)* $D State }

                $crate::macros::create_delegate!{#impl
                    [$D State] [$original]
                    [$mod] [$D ($D gen)*] [$trait] [$ctx]
                    $datause $data $datac
                    $impl
                    $(impl-extra: $impl_extra)?
                }
            }
        }
    };

    (#impl-fn
     [$Self:ident]
     [$trait:ident]
     Use: [$([$($use:tt)*])*]
     [$ctx:ident]
     [$fn:ident]
     Data: [$($data:tt)*] [$($datac:tt)*]
     Args: [$($args:tt)*] [$($params:tt)*]
    ) => {
        fn $fn(
            &mut self,
            conn: &::wayland_client::Connection,
            qh: &::wayland_client::QueueHandle<Self>,
            $($data)*
            $($args)*
        ) {
            $($($use)*)*

            <$Self as $trait>::$fn(
                self,
                $ctx {
                    conn,
                    qh,
                    data: $($datac)*,
                },
                $($params)*
            );
        }
    };

    (#impl
     [$State:ty] [$original:path]
     [$mod:path] [$($gen:tt)*]
     [$trait:ident] [$ctx:ident]
     $datause:tt $data:tt $datac:tt
     {$(
         $fn:ident($($arg:ident : $argty:ty),*);
     )*}

    $(impl-extra: { $($impl_extra:tt)* })?
    ) => {
        impl $($gen)* $original for $State {
            $($($impl_extra)*)?

            $(
            $crate::macros::create_delegate!{#impl-fn
                [Self]
                [$trait]
                Use: [
                    [use $mod::{$ctx};]
                    [use $mod::{$trait};]
                    $datause
                ]
                [$ctx]
                [$fn]
                Data: $data $datac
                Args: [$($arg : $argty),*] [$($arg),*]
            }
            )*
        }
    };


    (#trait-fn
     [$Self:ident] [$ctx:ident]
     [$fn:ident]
     Args: [$($args:tt)*] [$([$params:ident])*]
    ) => {
        fn $fn(&mut self, ctx: $ctx<'_, $Self>, $($args)*) {
            _ = ctx;
            $(_ = $params;)*
        }
    };

    (#trait
     [$mod:path]
     [$trait:ident $(: $base:path)?] [$ctx:ident]
     {$(
         $fn:ident($($arg:ident : $argty:ty),*);
     )*}
     {$($trait_extra:tt)*}
    ) => {
        pub trait $trait: $($base +)? Sized {
            $($trait_extra)*
            $(
            $crate::macros::create_delegate!{#trait-fn
                [Self] [$ctx] [$fn]
                Args: [$($arg : $argty),*] [$([$arg])*]
            }
            )*
        }
    };
}

#[doc(hidden)]
pub(crate) use create_delegate;
