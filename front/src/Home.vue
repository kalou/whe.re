<template>
	<div id="home" class="homepage">
        <h1>Where</h1>
        <p>Experimental openstreetmap search</p>

        <h2>Pick a feature</h2>
        <router-link to="/post_box" class="poi">dropbox</router-link>
        <router-link to="/convenience" class="poi">convenience store</router-link>
        <router-link to="/toilets" class="poi">toilets</router-link>
        <router-link to="/drinking_water" class="poi">water fountains</router-link>
        <router-link to="/hotel" class="poi">hotels</router-link>
        <router-link to="/billiards" class="poi">billiards</router-link>

        <h2>Search for more</h2>
        <div class="poibox">
            <PoiSearch @looking="highlightPoi" @select="addPoi"
				:apiProxy=poiSearch />
        </div>

        <h2>Also check out <a href="https://sco.re">sco.re</a></h2>

        <p class="explanation">All data comes from <a href="https://www.openstreetmap.org">OpenStreetMap</a>. Features and routes are calculated from a small PBF extract covering only the SF Bay area for now. Routes are computed by walking, not taking into account forbidden or dangerous ways - it will send you out for a walk on the freeway. Do not trust itineraries. If you want access to the API, or need specific developments, feel free to use contact at whe dot re.</p>
	</div>
</template>

<script>
import PoiSearch from './components/PoiSearch.vue'
export default {
    name: 'Home',
    methods: {
        addPoi(poi) {
            this.$router.push({ path: '/' + poi });
        },
        highlightPoi() {
        },
        nodeSearch(q) {
            return fetch(process.env.VUE_APP_API + '/graph/search?lat='
                + this.map_lat + '&lon=' + this.map_lon + '&q=' + q)
        },
        poiSearch() {
            return fetch(process.env.VUE_APP_API + '/graph/pois?lat='
                + this.map_lat + '&lon=' + this.map_lon)
        },
    },
    components: {
		PoiSearch,
    },
}
</script>

<style scoped>
.homepage {
    text-align: center;
    display: block;
}
.explanation {
    font-size: 10px;
}
.poibox {
    width: 50%;
    display: block;
    margin: 0px auto;
}
.poi {
    margin : 10px;
}
</style>
