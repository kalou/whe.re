<template>
    <div class="search">
        <input type="text" class="search" v-model="query"
            placeholder="Enter a name or address"
            v-on:keyup.enter="onSubmit"
            v-on:keydown.down="onSelDown"
            v-on:keydown.up="onSelUp"
            v-on:keydown.tab="onSubmit"
            @input="onChange"
            v-on:keyup.esc="onCancel">
        <div class="suggested" :class="{hidden: !suggestActive}">
            <div v-for="(node, i) in results"
                 :key="i"
                 :ref="'su' + i"
                 class="suggested-entry"
                 @mouseover="suggestSelected = i"
                 :class="{selected: suggestSelected == i}"
                 @click="onSubmit">
                <NodeBox v-bind=node />
            </div>
        </div>
        <span class="mdi mdi-crosshairs" @click="getCurrent" />
    </div>
</template>

<script>
import NodeBox from './Node.vue';
export default {
    name: 'NodeSearch',
    props: {
        'apiProxy': Function
    },
    data() {
        return {
            query: '',
            suggestActive: true,
            suggestSelected: 0,
            results: [],
        }
    },
    watch: {
        // When user scrolls selections we can highlight stuff or react
        suggestSelected() {
            this.$emit('looking', this.results[this.suggestSelected]);
        }
    },
    methods: {
        // W?
        //onHoverSuggestion(what) {
        //    this.suggestSelected(what);
		//},
		getCurrent() {
			navigator.geolocation.getCurrentPosition(pos =>
                this.$emit('select', {'kind': ['curloc'],
                        'name': 'current location',
                        'lat': pos.coords.latitude,
                        'lon': pos.coords.longitude
                }), err => console.log("geo err " +err.code),
                {timeout: 5 * 1000}
                    //enableHighAccuracy: true}
            );
		},
        onSelDown() {
            this.suggestSelected++;
            this.suggestSelected %= this.results.length;
        },
        onSelUp() {
            this.suggestSelected = this.suggestSelected - 1;
            if (this.suggestSelected < 0) {
                this.suggestSelected = Math.max(0, this.results.length-1);
            }
        },
        onChange() {
            if (!this.query) {
                this.onCancel();
                return;
            }

            this.apiProxy(this.query.toLowerCase())
                .then((resp) => {
                    return resp.json();
                })
                .then((json) => {
                    this.results = json.nodes;
                    this.suggestSelected = 0;
                    this.$emit('looking', this.results[this.suggestSelected]);
                });
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
        NodeBox,
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
