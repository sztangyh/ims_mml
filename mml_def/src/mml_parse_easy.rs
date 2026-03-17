#[allow(unused)]
#[macro_export]
macro_rules! mml_easy {
    (@hm_tp) => { BTreeMap<String, String> };
    (@hm_tp < $lft:lifetime>) => {BTreeMap<&$lft str, &$lft str>};
    //(@arm $e:expr, $id:ident, $ty:ty)
    (@paras ($id:ident = $e:expr, $($tt:tt)*) -> $g1:tt $g2:tt ($($ci:ident=$ce:expr,)*) $g4:tt) 
    => {
        mml_easy!(@paras ($($tt)* ) -> $g1 $g2 ($($ci = $ce,)* $id=$e,) $g4);
    };

    (@paras ($id:ident:Option<$ty:ty>, $($tt:tt)*) -> $g1:tt ($($oid:ident:$oty:ty,)*) $g3:tt $g4:tt) 
    => {
        mml_easy!(@paras ($($tt)*) -> $g1 ($($oid:$oty,)* $id:$ty,) $g3 $g4);
    };

    (@paras ($id:ident:$ty:ty, $($tt:tt)*) -> ($($mid:ident:$mty:ty,)*) $g2:tt $g3:tt $g4:tt) 
    => {
        mml_easy!(@paras ($($tt)* ) -> ($($mid:$mty,)* $id:$ty,) $g2 $g3 $g4);
    };

    ($mml:ident $(<$lft:lifetime>)?, $($tt:tt)+) 
    => {
        mml_easy!(@paras ($($tt)+,) -> () () () ($mml $(<$lft>)?));
    };
    
    (@paras ($(,)* ) ->
        ($($mid:ident:$mty:ty,)*)
        ($($oid:ident:$oty:ty,)*)
        ($($ci:ident = $e:expr,)*)
        ($mml:ident $(<$lft:lifetime>)?)
    ) => {
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone)]
            /// 由宏生成的目标 MML 结构体。
            pub struct $mml $(<$lft>)? {
                $(
                    pub $mid: $mty,
                )*
                $(
                    pub $oid: Option<$oty>,
                )*
            }

            impl $(<$lft>)? $mml $(<$lft>)? {
                #[allow(unused)]
                /// 解析参数体，若条件不满足或缺少必填字段则返回 `None`。
                pub fn from_mml_body(
                    body: &$($lft)? str
                ) -> Option<Self> {
                    $(
                        let mut $mid: Option<$mty> = None;
                    )*
                    $(
                        let mut $oid: Option<$oty> = None;
                    )*
                    $(
                        let mut $ci = false;
                    )*
                    let mut paras = mml_paras_iter(body);
                    while let Some((pname, pval)) = paras.next(){
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($ci)) {
                                $ci = true;
                                if pval != $e {return None;}
                            } else 
                        )*
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($mid)) {
                                $mid = Some(<$mty>::from_para(pval).ok()?);
                            } else
                        )*
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($oid)) {
                                $oid = Some(<$oty>::from_para(pval).ok()?);
                            } else
                        )*
                        {continue}
                    }
                    let chk = $($ci &&)* true;
                    if !chk {return None;}
                    Some(
                        $mml{
                            $(
                                $mid:$mid?,
                            )*
                            $(
                                $oid:$oid,
                            )*
                        }
                    )
                }
            }
    };
}

pub use mml_easy;



