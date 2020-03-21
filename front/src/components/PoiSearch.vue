<template>
    <div class="search">
        <input type="text" class="search" v-model="query"
            placeholder="Search for a point of interest.."
            v-on:keyup.enter="onSubmit"
            v-on:keydown.down="onSelDown"
            v-on:keydown.up="onSelUp"
            v-on:keydown.tab="onSubmit"
            @input="onChange"
            v-on:keyup.esc="onCancel">
        <div class="suggested" :class="{hidden: !suggestActive}">
            <div v-for="(poi, i) in results"
                 :key="i"
                 :ref="'su' + i"
                 class="suggested-entry"
                 @mouseover="suggestSelected = i"
                 :class="{selected: suggestSelected == i}"
                 @click="onSubmit">
                <PoiBox :name="poi" />
            </div>
        </div>
    </div>
</template>

<script>
import PoiBox from './Poi.vue';
export default {
    name: 'PoiSearch',
    props: {
        'apiProxy': Function
    },
    data() {
        return {
            query: '',
            suggestActive: true,
            suggestSelected: 0,
            suggested: [],
            results: [],
            poi_types: [],
        }
    },
    watch: {
        suggestSelected() {
            this.$emit('looking', this.results[this.suggestSelected]);
        }
    },
    methods: {
        filter_poi_types() {
            this.results = this.poi_types
                .filter(x => x.startsWith(this.query))
                .slice(0, 5);
        },
        get_poi_types() {
            if (!this.poi_types.length) {
                this.apiProxy()
                    .then((resp) => {
                        return resp.json();
                    })
                    .then((json) => {
                        this.poi_types = json['items'];
                        this.filter_poi_types();
                    });
            }
            this.filter_poi_types();
        },
        //XXX?
        //onHoverSuggestion(what) {
        //    this.suggestSelected(what);
        //},
        onSelDown() {
            this.suggestSelected++;
            this.suggestSelected %= this.results.length;
        },
        onSelUp() {
            this.suggestSelected = Math.max(0, this.suggestSelected - 1);
        },
        onChange() {
            if (!this.query) {
                this.onCancel();
                return;
            }

            this.get_poi_types(this.query.toLowerCase());
        },
        onSubmit() {
            if (this.results[this.suggestSelected]) {
                this.$emit('select', this.results[this.suggestSelected]);
            }
            this.onCancel();
        },
        onCancel() {
            this.$emit('looking', null);
            this.query = '';
            this.results = [];
            this.matching_pois = [];
            this.suggestSelected = 0;
        }
    },
    components: {
        PoiBox
    }
};
</script>

<style scoped>
    .search {
        display: block;
    }
    .search input {
        font-size: 14px;
        background-image: url("../../public/search.svg");
        background-size: 14px 14px;
        background-repeat: no-repeat;
        background-position: 10px, center;

        padding-left: 40px;

        -webkit-appearance: none;
        outline: none;
        border: none;
    }

    .suggested {
        -webkit-transition: height 500ms;
        overflow: hidden;
        display: inline-block;
        text-align: left;
    }

	.suggested.hidden {
        -webkit-transition: height 500ms;
		height: 0px;
	}

    .suggested-entry.selected {
        background-color: #cccccc;
    }

    .suggested-poi.selected {
        background-color: #cccccc;
    }

    .suggested-entry:hover {
        background-color: #cccccc;
    }
</style>
