macro_rules! _json_decode_count {
    () => { 0 };
    ($ident:ident $($next:ident)*) => { 1+_json_decode_count!($($next)*) };
}

macro_rules! _json_decode_variant_constructor {
    ($dec:expr, $name:ident, $variant:ident,) => {
        $name::$variant
    };
    ($dec:expr, $name:ident, $variant:ident, $($field:ident)*) => {{
        $(
            let $field = try!(::rustc_serialize::Decodable::decode($dec));
        )*
        $name::$variant( $($field),* )
    }};
}

#[macro_export]
macro_rules! json_enum_decoder {
    ($name:ident { $(
        $variant:ident ( $($fname:ident),* ),
    )* }) => {
        impl ::rustc_serialize::Decodable for $name {
            fn decode<D: ::rustc_serialize::Decoder>(d: &mut D)
                -> Result<$name, D::Error>
            {
                d.read_seq(|d, len| {
                    let s = try!(d.read_str());
                    match &s[..] {
                        $(
                            stringify!($variant) => {
                                let cnt = _json_decode_count!($($fname)*) + 1;
                                if cnt != len {
                                    return Err(d.error("Bad tuple length"));
                                }
                                Ok(_json_decode_variant_constructor!(
                                    d, $name, $variant, $($fname)*))
                            }
                        )*
                        _ => Err(d.error("unknown constructor name")),
                    }
                })
            }
        }
    }
}

