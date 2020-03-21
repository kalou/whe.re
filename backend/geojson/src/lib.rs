use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Serialize as AutoSerialize;

// The data part of the geojson walk lines
#[derive(Debug, AutoSerialize)]
pub struct GeoPoint(pub f64, pub f64);

#[derive(Debug, AutoSerialize)]
pub struct MultiLineSegment(pub GeoPoint, pub GeoPoint);

#[derive(Debug)]
pub struct MultiLine {
    pub data: Vec<MultiLineSegment>
}

impl Serialize for MultiLine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MultiLine", 3)?;
        state.serialize_field("type", "MultiLineString");
        state.serialize_field("coordinates", &self.data);
        state.end()
    }
}

#[derive(Debug)]
pub struct MultiPoint {
    pub data: Vec<GeoPoint>,
}

impl Serialize for MultiPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MultiPoint", 3)?;
        state.serialize_field("type", "MultiPoint");
        state.serialize_field("coordinates", &self.data);
        state.end()
    }
}

#[derive(Debug)]
pub struct Point {
    pub data: GeoPoint,
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Point", 3)?;
        state.serialize_field("type", "Point");
        state.serialize_field("coordinates", &self.data);
        state.end()
    }
}

#[derive(Debug, AutoSerialize)]
pub struct Properties {
    pub name: Option<String>,
    pub node_id: Option<i64>,
}

#[derive(Debug)]
pub struct Feature<T: Serialize> {
    pub properties: Properties,
    pub data: T, // Point, MultiPoint, MultiLine
}

impl<T: Serialize> Serialize for Feature<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GeoJson", 3)?;
        state.serialize_field("type", "Feature");
        state.serialize_field("properties", &self.properties);
        state.serialize_field("geometry", &self.data);
        state.end()
    }
}

#[derive(Debug)]
pub struct FeatureCollection<T: Serialize> {
    pub features: Vec<Feature<T>>,
}

impl<T: Serialize> Serialize for FeatureCollection<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GeoJson", 3)?;
        state.serialize_field("type", "FeatureCollection");
        state.serialize_field("features", &self.features);
        state.end()
    }
}
