<template>
    <div class="constraint">
        <div v-if="kind === 'Near'">
            {{cost/60}} minutes from <Node v-bind=from />
        </div>
        <div v-if="kind === 'NearPoi'">
            {{cost/60}} minutes from a <Poi :name=from />
        </div>
        <div v-if="kind === 'OnTheWay'">
            On the way from <Node v-bind=from /> to <Node v-bind=to />
        </div>
    </div>
</template>

<script>
import Node from './Node';
import Poi from './Poi';
export default {
    name: 'Constraint',
    props: ['kind', 'from', 'to', 'cost'],
    methods: {
        updateValue(evt) {
            // Used by additional v-if below select
            this.selectedMode = evt.target.value;
            this.emitConstraint();
        },
        onSelectNode(node) {
            console.log('submitNode ' + node.node_id);
            this.toNode = node;
            this.emitConstraint();
        },
        emitConstraint() {
            this.$emit('update', {
                'kind': this.selectedMode,
                'to': this.toNode
            });
        }
    },
    components: {
        Node,
        Poi
    },
}
</script>

<style scoped>
</style>
