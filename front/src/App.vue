<template>
	<div id="app">
        <div id=panel class=panel v-bind:class="{ active: showPanel }">
            <span class="mdi mdi-duck" @click=" showPanel = !showPanel "/>
            <ConstraintPicker @looking="highlightNode"
             :constraints=constraints
             :errmsg=errmsg
             @add="addConstraint"
             @remove="removeConstraint"
             :poiSearch="poiSearch"
             :nodeSearch="nodeSearch" />
		</div>
        <Map :lat=map_lat :lon=map_lon :zoom=map_zoom
             @move="onMapMove"
            :highlighted_node="hled"
            :nodes="map_nodes"
            :geojson="geojson"
            :heatmap_data="heatmap_data" />
	</div>
</template>

<script>
import ConstraintPicker from './components/ConstraintPicker.vue'
import Map from './components/Map.vue'

export default {
    name: 'App',
    data() {
        return {
            showPanel: true,
            errmsg: "",
            // Selected by user
            constraints: [],
            // Highlighted on map
            hled: null,
            map_nodes: [],
            geojson: {
                'type': 'featureCollection',
                'features': []
            },
            heatmap_data: {},
            // These are current
            map_lat: this.$route.query.lat || 37.781094,
            map_lon: this.$route.query.lon || -122.459261,
            map_zoom: 14
        }
    },
    methods: {
        reduce() {
            this.isActive = !this.isActive;
        },
        onMapMove(lonlat, zoom) {
            console.log("Map moved to lat=" + this.map_lat +
                        " lon=" + this.map_lon + " zoom " + zoom);
            this.$route.query.lat = this.map_lat;
            this.$route.query.lon = this.map_lon;
            /*
            this.map_lat = lonlat.lat;
            this.map_lon = lonlat.lng;
            this.map_zoom = zoom;
            */
        },
        nodeSearch(q) {
            return fetch(process.env.VUE_APP_API + '/graph/search?lat='
                + this.map_lat + '&lon=' + this.map_lon + '&q=' + q)
        },
        poiSearch() {
            return fetch(process.env.VUE_APP_API + '/graph/pois?lat='
                + this.map_lat + '&lon=' + this.map_lon)
        },
        highlightNode(node) {
            this.hled = node;
        },
        heatmap() {
            var req = {
                "pois": ["supermarket", "park"],
                "near": [
                    // Downtown
                    [{"Node": {"node_id": 6368211241}}, 5000]
                ]
            };
            fetch(process.env.VUE_APP_API + '/graph/score', {
                    'method': 'POST',
                    'headers': {'Content-Type': 'application/json'},
                    'body': JSON.stringify(req)
            }).then(response => {
                return response.json();
            }).then((res) => {
                this.heatmap_data = {
                    data: res.squares
                };
            });
        },
        path() {
            console.log("path with nodes" + this.nodes);
            var node1 = this.nodes[0];
            var node2 = this.nodes[1];
            fetch(process.env.VUE_APP_API + '/graph/path?from='
                + node1.node_id + '&to=' + node2.node_id 
                + '&via=school'
            ).then(response => {
                    return response.json();
                }).then((res) => {
                    this.geojson = res;
                });
        },
        addConstraint(c) {
            console.log("constraint added " + 
                JSON.stringify(c, 2, null));
            this.constraints.push(c);
            this.isochrone();
        },
        removeConstraint(removed) {
            // We use a key to identify which one?
            this.constraints.splice(removed, 1);
            this.isochrone();
        },
        to_nodespec(x) {
            if (typeof(x.node_id) == "number") {
                return {"Node": {"node_id": x.node_id}};
            } else {
                return {"LonLat": [x.lon, x.lat]}
            }
        },
        to_constraint(x) {
            //Adapt component constraint for API call
            switch(x.kind) {
                case "Near":
                    return {
                        "Near": [
                            this.to_nodespec(x.from),
                            x.cost
                        ]
                    };
                case "NearPoi":
                    return {
                        "NearPoi": [
                            x.from,
                            x.cost
                        ]
                    };
                case "OnTheWay":
                   return {
                        "OnTheWay": [
                            this.to_nodespec(x.from),
                            this.to_nodespec(x.to),
                        ]
                    };
            }
        },
        isochrone() {
            this.errmsg = "";
            console.log("isochrone with nodes " + this.nodes);
            this.geojson = {
                'type': 'featureCollection',
                'features': []
            };

            var req = {
                "poi": this.$route.params['poi'],
                "constraints": this.constraints.map(x =>
                    this.to_constraint(x))
            };

            console.log("req " + JSON.stringify(req, null, 2));

            // Spare a call and an error message when removing the last
            if (this.constraints.length == 0) {
                this.map_nodes = [];
                this.geojson = {};
                return;
            }

            fetch(process.env.VUE_APP_API + '/graph/isochrone', {
                    'method': 'POST',
                    'headers': {'Content-Type': 'application/json'},
                    'body': JSON.stringify(req)
            }).then(response => {
                return response.json();
            }).then((res) => {
                console.log('Got ' + JSON.stringify(res, null, 2));
                this.map_nodes = res.points.concat(res.nodes);
                this.geojson = res.paths;
                if (res.points.length == 0) {
                    this.errmsg = "No match, remove some criterias";
                }
            });
        },
    },
    components: {
		Map,
        ConstraintPicker
    },
}
</script>

<style scoped>
    .panel.active {
        width: 25em;
        height: auto;
    }
    .panel {
        top: 30px;
        left: 30px;
        padding: 0.2em;
        border-radius: 10px;
        color: black;
        background-color: white;
        position: fixed;
        z-index: 5000;
        overflow: hidden;

        height: 15px;
        width: 15px;
    }
</style>
