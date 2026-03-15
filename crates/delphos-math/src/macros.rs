/// ```ignore
/// dual_combination!(macro, [x, y, z]);
/// => macro!(x, y);
/// => macro!(x, z);
/// => macro!(y, z);
///
/// dual_combination!(macro, [x, y, z, w]);
/// => macro!(x, y);
/// => macro!(x, z);
/// => macro!(x, w);
/// => macro!(y, z);
/// => macro!(y, w);
/// => macro!(z, w);
/// ```
macro_rules! dual_combination {
    ($m:ident, [$start:tt, $($tail:tt),+ $(,)?]) => {
        $crate::macros::dual_combination!([$m] branch [$start] [$([$tail])+]);
    };

    ([$m:ident] $_:ident [$start:tt] []) => {};

    ([$m:ident] branch [$start:tt] [[$item:tt] $([$tail:tt])*]) => {
        $crate::macros::dual_combination!([$m] no_branch [$start] [$([$tail])*]);
        $crate::macros::dual_combination!([$m] branch [$item] [$([$tail])*]);
        $m!($start, $item);
    };

    ([$m:ident] no_branch [$start:tt] [[$item:tt] $([$tail:tt])*]) => {
        $crate::macros::dual_combination!([$m] no_branch [$start] [$([$tail])*]);
        $m!($start, $item);
    };
}

pub(crate) use dual_combination;
