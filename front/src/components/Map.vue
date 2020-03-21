<template>
  <div id="map0" class="map"></div>
</template>

<script>
import L from 'leaflet';
import icons from '../poi_fa.js';
export default {
    name: 'Map',
    data() {
        return {
            heatmap_layer: null
        }
    },
    props: {
        lat: Number,
        lon: Number,
        zoom: Number,
        // A different style for temporary marker
        highlighted_node: Object,
        // Displayed geojson feature
        geojson: Object,
        // Displayed nodes & pois
        nodes: Array,
        heatmap_data: Array,
    },
    watch: {
        // Remove or add marker
        highlighted_node() {
            if (this.temp_marker) {
                //this.map.removeLayer(this.temp_marker);
                this.temp_marker.remove();
            }
            if (this.highlighted_node) {
                this.temp_marker = this.marker_for_node(this.highlighted_node);
                this.temp_marker.openTooltip();
            }
        },
        nodes() {
            if (this.node_markers) {
                this.node_markers.forEach(x => x.removeFrom(this.map));
            }
            this.node_markers = this.nodes.filter(x => x)
                .map(x => {
                return this.marker_for_node(x);
            });
        },

        heatmap_data() {
            //console.log('New heatmap data '
            //    + JSON.stringify(this.heatmap_data));

            // Score is now max'ed at 100
            var scolor = (x) => {
                // Count the features
                var hex = Math.round(255 * x.score / 100)
                    .toString(16);
                return '#00' + ('0' + hex).slice(-2) + '00';
            }

            if (this.heatmap_layer) {
                console.log("cleaning up");
                this.heatmap_layer.removeFrom(this.map);
            }
            this.heatmap_layer = L.layerGroup().addTo(this.map)

            this.heatmap_data.map(x => {
                this.heatmap_layer.addLayer(
                    L.rectangle([[x.top, x.left], [x.bottom, x.right]], {
                    color: "#0000ff",
                    fillColor: scolor(x),
                    stroke: false,
                    fillOpacity: .5
                }).bindTooltip(x.features.toString()))
            });
        },
        geojson() {
            if (this.geojson_layer) {
                this.map.removeLayer(this.geojson_layer);
            }
            if (this.geojson) {
                this.geojson_layer = L.geoJSON(this.geojson, {
                    onEachFeature: this.geojsonDecorate,
                    style: {
                        weight: 2,
                    }
                }).addTo(this.map);
            }
        }
    },
    methods: {
        highlight(layer, v) {
            if (v) {
                layer.setStyle({
                    weight: 5,
                });
            } else {
                this.geojson_layer.resetStyle(layer);
            }
        },
        geojsonDecorate(feature, layer) {
            if (feature.properties && feature.properties.name) {
                layer.bindPopup(feature.properties.name);
            }
            layer.on('mouseover', e => this.highlight(e.target, true));
            layer.on('mouseout', e => this.highlight(e.target, false));
            layer.on('click', e => console.log('click ' + e.target));
        },

        update_mapview() {
            this.map.setView([this.lat, this.lon], this.zoom,
                {'animate': true})
        },
        nodeName(node) {
            if (node.name) {
                return node.name;
            } else if (node.kind.length) {
                return node.kind[0];
            } else {
                return "node " + node.node_id;
            }
        },
        marker_for_node(node) {
            if (!node) {
                return;
            }
            var icon = L.divIcon({
                iconSize: null,
                html: '<div><span class="mdi ' +
                    icons.by_kind(node.kind)
                + '" />' + this.nodeName(node) + '</div>'
            });
            var marker = L.marker([node.lat, node.lon],
                {title: this.nodeName(node),
                 icon: icon}).addTo(this.map);
            marker.bindTooltip(this.nodeName(node), {opacity: 0.7});
            return marker;
        },
        onMove() {
            var latlng = this.map.getCenter();
            this.$emit('move', latlng, this.map.getZoom());
        },
    },
    onClick(evt) {
        console.log('map click at ' + JSON.stringify(evt));
    },
    mounted() {
        this.map = L.map('map0', {
            'zoomControl': false,
            //'minZoom': 11,
            'maxZoom': 18
        });
        this.map.setView([this.lat, this.lon],
                this.zoom);
        this.map.on('moveend', this.onMove);
        this.map.on('click', this.onClick);
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        }).addTo(this.map);

        this.onMove();

        //L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', { attribution: 'Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community' }).addTo(this.map);
    }
}
</script>

<style scoped>
.map {
    height: 640px; 
}
@import '~leaflet/dist/leaflet.css';
</style>
