<template>
    <div class="conslist">
        <h1>Find the best place</h1>
        <span v-if="errmsg" class="errmsg">{{errmsg}}</span>
        <div :key="i" v-for="(cons, i) in constraints">
            <Constraint v-bind=cons />
            <button class="mdi mdi-minus" v-on:click="remove(i)" />
        </div>

        <div>
            <h2>Add a reference place</h2>
            Within
            <input type=number min="5" max="60" v-model="near_min"> minutes of
            <NodeSearch @select="onNear"
              @looking="(x) => $emit('looking', x)"
              :apiProxy="nodeSearch" />
        </div>

      <h2>Add points of interest</h2>
      <PoiSearch @select="onPoi" :apiProxy="poiSearch" />
   </div>
</template>

<script>
import Constraint from './Constraint.vue';
import NodeSearch from './NodeSearch';
import PoiSearch from './PoiSearch';
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
		onPoi(kind) {
			this.$emit('add', {
                'kind': 'NearPoi',
                'cost': 1500,
                'from': kind
			});
		},
        remove(i) {
            this.$emit('remove', i);
        },
    },
    components: {
        Constraint,
        NodeSearch,
        PoiSearch,
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
