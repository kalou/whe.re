/* Save some memory and get rid of tags.. Autogen ?*/

enum Poi {
    Bar,
    Parking,
    Billiards,
    Unknown
}


fn poi_type(s: &str) -> Poi {
    match s {
        "bar" | "pub" => Poi::Bar,
        "billiards" => Poi::Billiards,
        _ => Poi::Unknown
    }
}
