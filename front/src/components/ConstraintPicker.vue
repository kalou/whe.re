<template>
    <div class="conslist">
        <h1>Looking for {{this.$route.params['poi']}}</h1>
        <span v-if="errmsg" class="errmsg">{{errmsg}}</span>
        <div :key="i" v-for="(cons, i) in constraints">
            <Constraint v-bind=cons />
            <button class="mdi mdi-minus" v-on:click="remove(i)" />
        </div>

        <h2>On the way between</h2>
        <div class="addOnTheWay">
            <div v-if="from">
                <Node v-bind=from />
            </div>
            <div v-else>
                <NodeSearch @select="onOnTheWay"
                    @looking="(x) => $emit('looking', x)"
                    :apiProxy="nodeSearch" />
            </div>
            to
            <NodeSearch @select="onOnTheWay"
               @looking="(x) => $emit('looking', x)"
               :apiProxy="nodeSearch" />
        </div>

       <h2>Reachable from a place</h2>
        <div class="addNear">
        The {{this.$route.params['poi']}} should be within
            <input type=number min="5" max="60" v-model="near_min">
                minutes of <NodeSearch @select="onNear"
                              @looking="(x) => $emit('looking', x)"
                              :apiProxy="nodeSearch" />
        </div>
        <h2>Close to another feature</h2>
        The {{this.$route.params['poi']}} should also be within
        <div class="addNearPoi">
            <input type=number min="5" max="60" v-model="near_min_poi">
                minutes of a <PoiSearch @select="onNearPoi"
                                :apiProxy="poiSearch" />
        </div>
   </div>
</template>

<script>
import Constraint from './Constraint.vue';
import NodeSearch from './NodeSearch';
import PoiSearch from './PoiSearch';
import Node from './Node';
export default {
    name: 'ConstraintPicker',
    props: {
        'constraints': Array,
        'errmsg': String,
        'poiSearch': Function,
        'nodeSearch': Function
    },
    data() {
		return {
			selectedMode: 'cost',
            near_min: 15,
            near_min_poi: 5,
            from: null,
            to: null
		}
    },
    methods: {
		onNear(node) {
			this.$emit('add', {
                'kind': 'Near',
                'cost': 60 * this.near_min,
                'from': node
			});
		},
		onNearPoi(kind) {
			this.$emit('add', {
                'kind': 'NearPoi',
                'cost': 60 * this.near_min_poi,
                'from': kind
			});
		},
        onOnTheWay(node) {
            if (this.from) {
                this.$emit('add', {
                    'kind': 'OnTheWay',
                    'from': this.from,
                    'to': node
                });
                this.from = null;
            } else {
                this.from = node;
            }
        },
        remove(i) {
            this.$emit('remove', i);
        },
    },
    components: {
        Constraint,
        NodeSearch,
        PoiSearch,
        Node
    }
}
</script>

<style scoped>
    .conslist {
        padding: 10px;
    }
    .errmsg {
        color: red;
    }
</style>
