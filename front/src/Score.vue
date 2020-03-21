<template>
	<div id="app">
        <div id=panel class=panel v-bind:class="{ active: showPanel }">
            <span class="mdi mdi-duck" @click=" showPanel = !showPanel "/>
            <ScoreConstraintPicker @looking="highlightNode"
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
import ScoreConstraintPicker from './components/ScoreConstraintPicker.vue'
import Map from './components/Map.vue'

export default {
    name: 'Score',
    data() {
        var map_lat = this.$route.query.lat || 37.781094;
        var map_lon = this.$route.query.lon || -122.459261;
        return {
            showPanel: true,
            errmsg: "",
            // Selected by user
            constraints: [
                {'kind': 'Near',
                    'from': {'name': 'map_center',
                        'kind': ['curloc']},
                    'cost': 60*25},
                {'kind': 'NearPoi', 'from': 'billiards',
                    'cost': 60*25},
                {'kind': 'NearPoi', 'from': 'laundry',
                    'cost': 60*25},
                {'kind': 'NearPoi', 'from': 'supermarket',
                    'cost': 60*25}
            ],
            // Highlighted on map
            hled: null,
            map_nodes: [],
            geojson: {
                'type': 'featureCollection',
                'features': []
            },
            heatmap_data: [],
            // These are current
            map_lat: map_lat,
            map_lon: map_lon,
            map_zoom: 14
        }
    },
    methods: {
        reduce() {
            this.isActive = !this.isActive;
        },
        onMapMove(lonlat, zoom) {
            this.map_lat = lonlat.lat;
            this.map_lon = lonlat.lng;

            console.log("Map moved to lat=" + this.map_lat +
                " lon=" + this.map_lon + " zoom " + zoom);

            if (this.constraints.some(x => x.kind == 'Near' &&
                    x.from.name == 'map_center'))
                this.heatmap();
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
                constraints: this.constraints.map(x => this.to_constraint(x))
            }
            fetch(process.env.VUE_APP_API + '/graph/score', {
                    'method': 'POST',
                    'headers': {'Content-Type': 'application/json'},
                    'body': JSON.stringify(req)
            }).then(response => {
                return response.json();
            }).then((res) => {
                this.heatmap_data = res.squares
            });
        },
        addConstraint(c) {
            console.log("constraint added " + 
                JSON.stringify(c, 2, null));
            this.constraints.push(c);
            this.heatmap();
        },
        removeConstraint(removed) {
            // We use a key to identify which one?
            this.constraints.splice(removed, 1);
            this.heatmap();
        },
        to_nodespec(x) {
            if (x.name == 'map_center') {
                return {"LonLat": [this.map_lon, this.map_lat]}
            }
            if (typeof(x.node_id) == Number) {
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
    },
    components: {
		Map,
        ScoreConstraintPicker
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
