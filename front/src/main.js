import Vue from 'vue'
import VueRouter from 'vue-router'

import App from './App.vue'
import Score from './Score.vue'
import Home from './Home.vue'

Vue.config.productionTip = false

import L from 'leaflet';
console.log(L.Marker.prototype.options.icon);

delete L.Icon.Default.prototype._getIconUrl;
L.Icon.Default.mergeOptions({
    iconRetinaUrl: require('leaflet/dist/images/marker-icon-2x.png'),
    iconUrl: require('leaflet/dist/images/marker-icon.png'),
    shadowUrl: require('leaflet/dist/images/marker-shadow.png'),
});

Vue.use(VueRouter);

const routes = () => {
    switch(location.hostname) {
        case 'sco.re':
            return [{path: '/', component: Score}];
        default:
            return [
                {path: '/score', component: Score},
                {path: '/:poi', component: App},
                {path: '/', component: Home},
            ]
    }
}

const router = new VueRouter({
    routes: routes(),
    mode: 'history'
});

new Vue({
    router,
    render: c => c('router-view')
}).$mount('#app')
