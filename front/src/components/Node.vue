<template>
    <div class=node>
        <span class="mdi" :class=getIcon() />
        <span class="node-name">{{name}}</span>
        <span class="node-address">{{address}}</span>
    </div>
</template>

<script>
import icons from '../poi_fa.js';
export default {
    name: 'Node',
    methods: {
        getIcon() { return icons.by_kind(this.kind) },
    },
    props: {
        'name': String,
        'node_id': Number,
        'address': String,
        'kind': Array,
        'wheelchair': Boolean,
        'lat': Number,
        'lon': Number
    }
}
</script>

<style scoped>
    @import '~@mdi/font/css/materialdesignicons.min.css';
    .mdi {
        padding: 10px;
        color: grey;
    }
    .node {
        margin-top: 1px;
    }
    .node-address {
        margin: 5px;
    }
</style>
