use phf::phf_map;

pub static TYPES: phf::Map<&'static str, &'static str> = phf_map! {
    "serial" => "integer",
};
