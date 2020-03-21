use geojson;
use graph;

use std::rc::Rc;

use super::api::NodeDescription;

// The below is converters from structs to API Results
// especially Node description and geojson conversions
pub trait ApiFrom<T> {
    fn api_from(other: T) -> Self;
}

impl ApiFrom<Vec<&osm::Node>> for geojson::FeatureCollection<geojson::Point> {
    fn api_from(li: Vec<&osm::Node>) -> Self {
        geojson::FeatureCollection {
            features: li.into_iter()
                .map(|node| geojson::Feature::api_from(node))
                .collect()
        }
    }
}

impl ApiFrom<&osm::Node> for geojson::GeoPoint {
    fn api_from(node: &osm::Node) -> Self {
        geojson::GeoPoint(node.lon, node.lat)
    }
}

impl ApiFrom<&osm::Node> for geojson::Point {
    fn api_from(node: &osm::Node) -> Self {
        geojson::Point {
            data: geojson::GeoPoint::api_from(node)
        }
    }
}

impl ApiFrom<&osm::Node> for geojson::Feature<geojson::Point> {
    fn api_from(node: &osm::Node) -> Self {
        geojson::Feature {
            properties: geojson::Properties {
                node_id: Some(node.id),
                name: node.name().map(String::from),
            },
            data: geojson::Point::api_from(node)
        }
    }
}

impl<'a, W: 'a + graph::GraphWalker<'a, osm::Db>> ApiFrom<&graph::Step<'a, osm::Db, W>> for geojson::MultiLineSegment {
    fn api_from(step: &graph::Step<'a, osm::Db, W>) -> Self {
        let from_node = match &step.from {
            Some(prev_step) => prev_step.to,
            None => step.to
        };
        geojson::MultiLineSegment(
            geojson::GeoPoint::api_from(from_node),
            geojson::GeoPoint::api_from(step.to)
        )
    }
}

impl<'a, W: 'a + graph::GraphWalker<'a, osm::Db>> ApiFrom<graph::WalkResult<'a, osm::Db, W>> for geojson::MultiLine {
    fn api_from(res: graph::WalkResult<'a, osm::Db, W>) -> Self {
        geojson::MultiLine {
            data: res.steps.iter()
                .filter(|step| step.from.is_some())
                .map(|step| geojson::MultiLineSegment::api_from(step))
                .collect(),
        }
    }
}

impl<'a, W: 'a + graph::GraphWalker<'a, osm::Db>> ApiFrom<&'a Rc<graph::Step<'a, osm::Db, W>>> for geojson::MultiLine {
    fn api_from(step: &'a Rc<graph::Step<'a, osm::Db, W>>) -> Self {
        geojson::MultiLine {
            data: step.into_iter()
                .map(|prev| geojson::MultiLineSegment::api_from(prev))
                .collect()
        }
    }
}
