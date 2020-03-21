<template>
    <div class=poi>
        <span class="mdi" :class=getIcon()></span>{{name}}
    </div>
</template>

<script>
import icons from '../poi_fa.js';
export default {
    name: 'Poi',
    props: {
        'name': String,
    },
    methods: {
        getIcon() { return icons.by_name(this.name, true) },
    }
}
</script>

<style scoped>
    .mdi {
        padding: 10px;
        color: grey;
    }
    .poi {
        font-size: 1em;
        margin-top: 1px;
        color: blue;
    }
</style>
