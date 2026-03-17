//pub(crate) use FromPara;
//macro_rules! choose_tp_hm {
//}


#[allow(unused)]
#[macro_export]
macro_rules! mml_def {
    (@hm_tp ) => { std::collections::BTreeMap<String, String> };
    (@hm_tp < $lft:lifetime>) => {std::collections::BTreeMap<&$lft str, &$lft str>};
    //(@arm $e:expr, $id:ident, $ty:ty)
    (@paras ($id:ident:Ignore, $($tt:tt)*) ->$g1:tt $g2:tt $g3:tt ($($ig:ident,)*) $g5:tt $g6:tt $g7:tt) 
    => {
        mml_def!(@paras ($($tt)*) -> $g1 $g2 $g3 ($($ig,)* $id,) $g5 $g6 $g7);
    };

    (@paras ($id:ident:Absent, $($tt:tt)*) -> $g1:tt $g2:tt ($($abs:ident,)*) $g4:tt $g5:tt $g6:tt $g7:tt) 
    => {
        mml_def!(@paras ($($tt)* ) -> $g1 $g2 ($($abs,)* $id,) $g4 $g5 $g6 $g7);
    };

    (@paras ($id:ident:Mapped, $($tt:tt)*) -> $g1:tt $g2:tt $g3:tt $g4:tt ($($im:ident,)*) $g6:tt $g7:tt) 
    => {
        mml_def!(@paras ($($tt)* ) -> $g1 $g2 $g3 $g4 ($($im,)* $id,) $g6 (mapper));
    };

    (@paras ($id:ident:Option<$ty:ty>, $($tt:tt)*) -> $g1:tt ($($oid:ident:$oty:ty,)*) $g3:tt $g4:tt $g5:tt $g6:tt $g7:tt) 
    => {
        mml_def!(@paras ($($tt)*) -> $g1 ($($oid:$oty,)* $id:$ty,) $g3 $g4 $g5 $g6 $g7);
    };

    (@paras ($id:ident:$ty:ty, $($tt:tt)*) -> ($($mid:ident:$mty:ty,)*) $g2:tt $g3:tt $g4:tt $g5:tt $g6:tt $g7:tt) 
    => {
        mml_def!(@paras ($($tt)* ) -> ($($mid:$mty,)* $id:$ty,) $g2 $g3 $g4 $g5 $g6 $g7);
    };

    ($builder:ident, $mml:ident $(<$lft:lifetime>)?, $($tt:tt)+) 
    => {
        mml_def!(@paras ($($tt)+,) -> () () () () () ($builder, $mml $(<$lft>)?) () );
    };
    
    (@paras ($(,)* ) ->
        ($($mid:ident:$mty:ty,)*)
        ($($oid:ident:$oty:ty,)*)
        ($($abs:ident,)*)
        ($($ig:ident,)*)
        ($($im:ident,)*)
        ($builder:ident, $mml:ident $(<$lft:lifetime>)?)
        ($($mapper:ident)?)
    ) => {
            
            #[derive(Debug, Clone)]
            #[allow(unused)]
            #[allow(non_camel_case_types)]
            /// 由 `mml_def!` 生成的 Builder 类型。
            pub struct $builder {
                $(
                    pub $abs: String,
                )*
            }

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
                $(
                    pub $im: std::rc::Rc<str>,
                )*
                pub base: std::rc::Rc<$builder>,
                pub differs: mml_def!(@hm_tp $(<$lft>)?)
            }

            impl $(<$lft>)? $builder {
                #[allow(unused)]
                /// 根据外部缺省参数映射构建 Builder。
                pub fn new(hm: &std::collections::HashMap<String, String>) -> Self{
                    $(
                        let $abs = hm.get(stringify!($abs)).unwrap().clone();
                    )*
                    Self{
                        $($abs,)*
                    }
                }
                #[allow(unused)]
                /// 解析参数体并构建目标结构体。
                pub fn from_mml_body(
                    self:&std::rc::Rc<Self>, 
                    $($mapper: &mut std::collections::HashMap<std::rc::Rc<str>, std::rc::Rc<str>>,)? 
                    body: &$($lft)? str
                ) -> Result<$mml $(<$lft>)?, ParaParseError> {
                    $(
                        let mapper = $mapper;
                    )?
                    $(
                        let mut $mid: Option<$mty> = None;
                    )*
                    $(
                        let mut $oid: Option<$oty> = None;
                    )*
                    $(
                        let mut $abs: Option<&str> = None;
                    )*
                    $(
                        let mut $im: Option<std::rc::Rc<str>> = None;
                    )*
                    //println!("{}", concat!($(stringify!($ig)," "),*));
                    let mut differ = std::collections::BTreeMap::new();
                    let mut paras = mml_paras_iter(body);
                    while let Some((pname, pval)) = paras.next(){
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($mid)) {
                                let v = <$mty>::from_para(pval)
                                .map_err(|e|ParaParseError::InvalidFormat(
                                    format!("{}: {}", stringify!($mid), pval)))?;
                                $mid = Some(v);
                            } else
                        )*
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($oid)) {
                                let v = <$oty>::from_para(pval)
                                .map_err(|e|ParaParseError::InvalidFormat(
                                    format!("{}: {}", stringify!($oid), pval)))?;
                                $oid = Some(v);
                            } else
                        )*
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($abs)) { 
                                $abs = Some(pval); 
                            } else
                        )*
                        $(
                            if pname.eq_ignore_ascii_case(stringify!($im)) { 
                                if let Some((_k, v)) = mapper.get_key_value(pval) {
                                    $im = Some(v.clone())
                                }else{
                                    let v:std::rc::Rc<str> = std::rc::Rc::from(pval);
                                    mapper.insert(v.clone(), v.clone());
                                    $im = Some(v)
                                }
                            } else
                        )*
                        if $( !pname.eq_ignore_ascii_case(stringify!($ig)) &&)* true {
                            differ.insert(pname.into(), pval.into());
                        }
                    }
                    $(
                        if let Some(s) = $abs{
                            if s != self.$abs.as_str() {differ.insert(stringify!($abs).into(), s.into());}
                        }else {
                            differ.insert(stringify!($abs).into(), "".into());
                        }
                    )*
                    Ok(
                        $mml{
                            $(
                                $mid:$mid.ok_or(ParaParseError::NotFound(stringify!($mid).into()))?,
                            )*
                            $(
                                $oid:$oid,
                            )*
                            $(
                                $im:$im.ok_or(ParaParseError::NotFound(stringify!($im).into()))?,
                            )*
                            base: self.clone(),
                            differs: differ
                        }
                    )
                }
            }
    };
}

pub use mml_def;



